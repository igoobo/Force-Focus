// 파일 위치: src-tauri/src/backend_communicator.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{command, State};

// lib.rs에서 정의한 전역 상태 타입들
use crate::{
    ActiveSessionInfo,
    SessionStateArcMutex,
    StorageManagerArcMutex,
    InputStatsArcMutex,
};

use crate::Task;
// StorageManager의 메서드를 호출하기 위해 모듈 import
use crate::storage_manager; 
use std::time::{SystemTime, UNIX_EPOCH}; // 세션 시작 시간 생성용
use uuid::Uuid; // 로컬에서 임시 세션 ID 생성용

// 백그라운드 동기화를 위해 tokio::spawn과 Arc, Mutex를 사용
use tokio::spawn;
use std::sync::{Arc, Mutex};

use dotenv::dotenv;
use std::env;

// --- 1. 상수 정의 ---

fn get_api_base_url() -> String {
    dotenv().ok(); // .env 파일 로드 (없어도 패닉 안 남)
    // .env에 값이 없으면 로컬호스트를 기본값으로 사용 (개발 편의성)
    env::var("API_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000/api/v1".to_string())
}

// --- 2. API 요청/응답을 위한 구조체 ---

/// `POST /feedback` API의 Request Body 스키마
/// (API 명세서: event_id, feedback_type)
#[derive(Debug, Serialize)]
struct FeedbackRequest<'a> {
    event_id: &'a str, // 이벤트 ID (임시)
    feedback_type: &'a str, // "is_work" 또는 "distraction_ignored"
}

// (API 응답을 위한 Deserialize 구조체도 필요시 추가)
// #[derive(Debug, Deserialize)]
// struct FeedbackResponse { ... }

// --- 세션 API 요청/응답 모델 ---
#[derive(Debug, Serialize)]
struct SessionStartRequest<'a> {
    task_id: Option<&'a str>,
    goal_duration: u32,
}
#[derive(Debug, Deserialize)]
struct SessionStartResponse {
    session_id: String,
    start_time: String, // ISO 8601
}
#[derive(Debug, Serialize)]
struct SessionEndRequest {
    user_evaluation_score: u8,
}

// --- 3. BackendCommunicator 상태 정의 ---

/// reqwest::Client를 전역 상태로 관리하기 위한 구조체
/// Client는 내부에 Arc를 가지고 있어 복제(clone)에 저렴
pub struct BackendCommunicator {
    client: Client,
}

impl BackendCommunicator {
    /// 앱 시작 시 호출될 생성자
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

// --- 4. 이 모듈에 속한 Tauri 커맨드 정의 ---

// --- 로그인 커맨드 ---
#[command]
pub fn login(
    access_token: String,
    refresh_token: String,
    user_email: String,
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
) -> Result<(), String> {
    let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;
    
    // LSN에 토큰 저장
    storage_manager.save_auth_token(&access_token, &refresh_token, &user_email)?;
    
    println!("User logged in: {}", user_email);
    Ok(())
}

// --- 로그아웃 커맨드 ---
#[command]
pub fn logout(
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
) -> Result<(), String> {
    let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;
    
    // LSN에서 토큰 삭제
    storage_manager.delete_auth_token()?;
    
    println!("User logged out.");
    Ok(())
}


/// '개입'에 대한 사용자 피드백을 서버로 전송하는 비동기(async) 커맨드
///
/// # Arguments
/// * `feedback_type` - 프론트엔드에서 받은 피드백 (예: "is_work")
#[command]
pub async fn submit_feedback(
    feedback_type: String, 
    // [수정] 네트워크 클라이언트(BackendCommunicator) 대신 LSN(StorageManager) 주입
    storage_manager_mutex: State<'_, StorageManagerArcMutex> 
) -> Result<(), String> {
    
    // 1. 고유 이벤트 ID 생성
    let event_id = format!("event-{}", Uuid::new_v4()); 

    println!("Submitting feedback (to LSN): type={}", feedback_type);

    // 2. LSN 락 획득
    let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;
    
    // 3. 로컬 DB에 저장
    storage_manager.cache_feedback(&event_id, &feedback_type)?; 
    
    println!("Feedback cached successfully: {}", event_id);

    Ok(())
}


// --- 세션 시작 커맨드 ---
#[command]
pub async fn start_session(
    task_id: Option<String>,
    goal_duration: u32,
    // comm_state는 백그라운드 스레드로 'move'되어야 하므로 Arc로 감싸진 State
    comm_state: State<'_, Arc<BackendCommunicator>>,
    session_state_mutex: State<'_, SessionStateArcMutex>,
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
    input_stats_mutex: State<'_, InputStatsArcMutex>,
) -> Result<ActiveSessionInfo, String> { // React에 ActiveSessionInfo 반환


    // 1. '쓰기' 락 & 토큰 로드: .await 전에 LSN과 전역 상태 즉시 업데이트
    // 반환값: (ActiveSessionInfo, Option<String>) -> (세션정보, 토큰)
    let (info, auth_token) = {  // 락 범위를 제한하기 위해 새 스코프 생성
        let mut session_state = session_state_mutex.lock().map_err(|e| format!("State lock error: {}", e))?;
        let storage_manager = storage_manager_mutex.lock().map_err(|e| format!("Storage lock error: {}", e))?;

        let mut input_stats = input_stats_mutex.lock().map_err(|e| format!("InputStats lock error: {}", e))?;

        if session_state.is_some() {
            return Err("Session already active.".to_string());
        }

        // LSN에서 인증 토큰 로드 (로그인 상태 확인)
        // load_auth_token은 Result<Option<(access, refresh, email)>, String>을 반환한다고 가정
        let token = storage_manager.load_auth_token()
            .unwrap_or(None) // 에러나면 무시 (오프라인/미로그인으로 간주)
            .map(|t| t.0);   // (access_token, refresh_token, email) 중 access_token만 추출

        // 서버 응답을 기다리지 않고, 로컬에서 즉시 세션 정보를 생성
        let session_id = format!("local-{}", Uuid::new_v4());
        let start_time_s = SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?.as_secs();

        let info = ActiveSessionInfo {
            session_id: session_id.clone(),
            task_id: task_id.clone(), // task_id도 백그라운드 스레드로 move하기 위해 clone
            start_time_s,
        };
        
        // LSN에 저장
        storage_manager.save_active_session(&info)?;
        // 전역 상태 업데이트
        *session_state = Some(info.clone());

        // 새 세션이 시작될 때, '이벤트 횟수'를 0으로 초기화
        input_stats.meaningful_input_events = 0;
        input_stats.last_meaningful_input_timestamp_ms = start_time_s * 1000;
        input_stats.last_mouse_move_timestamp_ms = start_time_s * 1000;

        eprintln!("Session started (Offline-First). ID: {}", info.session_id);

        (info, token) // info와 token을 튜플로 반환
        // MutexGuard('session_state', 'storage_manager')는 여기서 자동으로 drop (락 해제)
    };

    // 2. 백그라운드 동기화: UI(React)를 기다리게 하지 않고,
    //    별도 스레드에서 '느린' 네트워크 작업을 수행
    // 토큰이 존재할 때만 서버 동기화를 시도합니다.
    if let Some(token) = auth_token {
        let comm_state_clone = comm_state.inner().clone(); 
        let info_clone = info.clone();

        spawn(async move {
            let task_id_ref = info_clone.task_id.as_deref();
            let request_body = SessionStartRequest {
                task_id: task_id_ref,
                goal_duration,
            };
            let url = format!("{}/sessions/start", get_api_base_url());

            println!("Background sync: Attempting to sync session {} to server...", info_clone.session_id);
            
            // MOCK_AUTH_TOKEN 대신 진짜 token 사용
            match comm_state_clone.client.post(&url).bearer_auth(token).json(&request_body).send().await {
                Ok(response) if response.status().is_success() => {
                    // 성공 시 서버 응답(Server Session ID) 처리 로직이 필요할 수 있음
                    println!("Background sync: Session {} synced successfully.", info_clone.session_id);
                }
                _ => {
                    eprintln!("Background sync: Failed to sync session {}. Will retry later.", info_clone.session_id);
                }
            }
        });
    } else {
        println!("Background sync skipped: No auth token found (User not logged in or offline).");
    }

    Ok(info)
}

// --- 세션 종료 커맨드 ---
#[command]
pub async fn end_session(
    user_evaluation_score: u8,
    // comm_state도 백그라운드 동기화를 위해 Arc<BackendCommunicator>를 받도록 변경
    comm_state: State<'_, Arc<BackendCommunicator>>,
    session_state_mutex: State<'_, SessionStateArcMutex>,
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
) -> Result<(), String> {

    // 1. '쓰기' 락: .await 전에 LSN과 전역 상태를 즉시 업데이트
    // 반환값: (session_id, auth_token)
    let (active_session_id, auth_token) = {
        let mut session_state = session_state_mutex.lock().map_err(|e| e.to_string())?;
        let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;
        
        let active_session_id = session_state.as_ref()
            .map(|s| s.session_id.clone())
            .ok_or_else(|| "No active session to end.".to_string())?;

        // 백그라운드 동기화를 위해 LSN에서 토큰 읽기
        let token = storage_manager.load_auth_token()
            .unwrap_or(None)
            .map(|t| t.0);

        // LSN 및 전역 상태 정리 (먼저 실행)
        storage_manager.delete_active_session()?;
        *session_state = None; // 전역 상태 초기화

        println!("Session ID {} successfully ended locally (score: {}).", active_session_id, user_evaluation_score);
        (active_session_id, token) // 락 해제 전 데이터 복사
        // MutexGuard('session_state', 'storage_manager')는 여기서 자동으로 drop (락 해제)
    };

    // 2. 백그라운드 동기화: UI를 기다리게 하지 않음
    if let Some(token) = auth_token {
        //  환경 변수 URL 사용
        let url = format!("{}/sessions/{}", get_api_base_url(), active_session_id);
        
        let request_body = SessionEndRequest { user_evaluation_score };
        let comm_state_clone = comm_state.inner().clone(); 

        spawn(async move {
            println!("Background sync: Attempting to sync session end for {}", active_session_id);
            
            // MOCK_AUTH_TOKEN 대신 진짜 token 사용
            let _ = comm_state_clone.client.put(&url).bearer_auth(token).json(&request_body).send().await
                .map_err(|e| eprintln!("Background sync: Warning: Failed to sync session end for {}: {}", active_session_id, e));
                
            println!("Background sync: Session end sync attempt finished for {}.", active_session_id);
        });
    } else {
        println!("Background sync skipped: No auth token found (User not logged in or offline).");
    }

    // 3. 즉시 반환
    Ok(())
}


// --- [추가] Task 3.5: '타이머 위젯' 동기화를 위한 'PULL' API ---
// [!] (비동기 아님) 이 함수는 '읽기'만 수행하므로 매우 빠릅니다.
#[command]
pub fn get_current_session_info(
    session_state_mutex: State<'_, SessionStateArcMutex>
) -> Result<Option<ActiveSessionInfo>, String> {
    let session_state = session_state_mutex.lock().map_err(|e| format!("State lock error: {}", e))?;
    
    // [!] 전역 상태('SessionState')를 복제(clone)하여 React로 반환
    Ok(session_state.clone())
}


// --- ['get_tasks' 커맨드 (빌드 오류 수정용) ---
// MainView.tsx의 'fetch'를 'invoke'로 대체하기 위한 Rust 커맨드.
// [!] (임시) handlers.ts의 'mockTasks' 데이터를 Rust에 하드코딩
#[command]
pub fn get_tasks() -> Result<Vec<Task>, String> {
    println!("Rust command 'get_tasks' invoked (returning mock data)");
    
    // handlers.ts의 mockTasks 데이터를 Rust로 변환
    let mock_tasks = vec![
        Task {
            id: "task-coding-session".to_string(),
            user_id: "desktop-user-123".to_string(),
            task_name: "코딩 세션 진행".to_string(),
            description: "Force-Focus 데스크톱 앱 프런트엔드 개발".to_string(),
            due_date: "2023-12-31T23:59:59Z".to_string(),
            status: "active".to_string(),
            target_executable: "vscode.exe".to_string(),
            target_arguments: vec![],
            created_at: "2023-10-26T10:00:00Z".to_string(),
            updated_at: "2023-10-26T10:00:00Z".to_string(),
        },
        Task {
            id: "task-report-writing".to_string(),
            user_id: "desktop-user-123".to_string(),
            task_name: "주간 보고서 작성".to_string(),
            description: "지난 주 작업 내용 정리 및 보고서 초안 작성".to_string(),
            due_date: "2023-11-03T18:00:00Z".to_string(),
            status: "pending".to_string(),
            target_executable: "word.exe".to_string(),
            target_arguments: vec![],
            created_at: "2023-10-25T09:00:00Z".to_string(),
            updated_at: "2023-10-25T09:00:00Z".to_string(),
        },
    ];

    Ok(mock_tasks)
}