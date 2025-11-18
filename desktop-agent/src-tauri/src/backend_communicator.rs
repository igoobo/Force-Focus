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


// --- 1. 상수 정의 ---

// MSW 목업이 아닌, 실제 FastAPI 백엔드 서버의 주소
// (로컬 개발 환경을 가정)
const API_BASE_URL: &str = "http://127.0.0.1:8000/api/v1"; 

// '가상 사용자' 전략을 위한 하드코딩된 인증 토큰
// msw/handlers.ts에 정의된 'mock-jwt-token-123'
const MOCK_AUTH_TOKEN: &str = "mock-jwt-token-123";

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

/// '개입'에 대한 사용자 피드백을 서버로 전송하는 비동기(async) 커맨드
///
/// # Arguments
/// * `feedback_type` - 프론트엔드에서 받은 피드백 (예: "is_work")
/// * `comm_state` - Tauri가 주입하는 BackendCommunicator 전역 상태
#[command]
pub async fn submit_feedback(
    feedback_type: String, 
    comm_state: State<'_, BackendCommunicator>
) -> Result<(), String> {
    
    // (임시) 현재는 event_id가 없으므로 임의의 값을 사용
    let event_id = "temp_event_001"; 

    let request_body = FeedbackRequest {
        event_id,
        feedback_type: &feedback_type,
    };

    let url = format!("{}/feedback", API_BASE_URL);

    println!( // 디버깅 로그
        "Submitting feedback to {}: type={}",
        url, request_body.feedback_type
    );

    match comm_state.client
        .post(&url)
        .bearer_auth(MOCK_AUTH_TOKEN) // '가상 사용자' 토큰 사용
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                println!("Feedback submitted successfully.");
                Ok(())
            } else {
                let error_msg = format!("API Error: {}", response.status());
                eprintln!("{}", error_msg);
                Err(error_msg)
            }
        }
        Err(e) => {
            let error_msg = format!("Reqwest Error: {}", e);
            eprintln!("{}", error_msg);
            Err(error_msg)
        }
    }
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


    // 1. '쓰기' 락: .await 전에 LSN과 전역 상태를 즉시 업데이트
    let info = { // 락 범위를 제한하기 위해 새 스코프 생성
        let mut session_state = session_state_mutex.lock().map_err(|e| format!("State lock error: {}", e))?;
        let storage_manager = storage_manager_mutex.lock().map_err(|e| format!("Storage lock error: {}", e))?;

        let mut input_stats = input_stats_mutex.lock().map_err(|e| format!("InputStats lock error: {}", e))?;

        if session_state.is_some() {
            return Err("Session already active.".to_string());
        }

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

        info // 이 info를 스코프 밖으로 반환
        // MutexGuard('session_state', 'storage_manager')는 여기서 자동으로 drop (락 해제)
    };

    // 2. 백그라운드 동기화: UI(React)를 기다리게 하지 않고,
    //    별도 스레드에서 '느린' 네트워크 작업을 수행

    
    // comm_state(Arc)를 백그라운드 스레드로 move
    let comm_state_clone = comm_state.inner().clone(); 

    let info_clone = info.clone();

    spawn(async move {

        // 'info_clone' (소유권 O)의 데이터를 빌려쓰므로 'static 수명 문제 해결
        let task_id_ref = info_clone.task_id.as_deref();
        let request_body = SessionStartRequest {
            task_id: task_id_ref,
            goal_duration,
        };
        let url = format!("{}/sessions/start", API_BASE_URL);
        
        println!("Background sync: Attempting to sync session {} to server...", info_clone.session_id);
        match comm_state_clone.client.post(&url).bearer_auth(MOCK_AUTH_TOKEN).json(&request_body).send().await {
            Ok(response) if response.status().is_success() => {
                let response_body: SessionStartResponse = response.json().await.unwrap(); // 간단한 unwrap
                println!("Background sync: Session {} synced successfully. Server ID: {}", info_clone.session_id, response_body.session_id);
                // (여기서 LSN의 local- ID를 Server ID로 업데이트하는 로직이 필요할 수 있음
            }
            _ => {
                eprintln!("Background sync: Failed to sync session {}. Will retry later.", info_clone.session_id);
            }
        }
    });

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
    let active_session_id = {
        let mut session_state = session_state_mutex.lock().map_err(|e| e.to_string())?;
        let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;
        
        let active_session_id = session_state.as_ref()
            .map(|s| s.session_id.clone())
            .ok_or_else(|| "No active session to end.".to_string())?;

        // LSN 및 전역 상태 정리 (먼저 실행)
        storage_manager.delete_active_session()?;
        *session_state = None; // 전역 상태 초기화

        println!("Session ID {} successfully ended locally (score: {}).", active_session_id, user_evaluation_score);
        active_session_id // 스코프 밖으로 ID 반환
        // MutexGuard('session_state', 'storage_manager')는 여기서 자동으로 drop (락 해제)
    };

    // 2. 백그라운드 동기화: UI를 기다리게 하지 않음
    let url = format!("{}/sessions/{}", API_BASE_URL, active_session_id);
    let request_body = SessionEndRequest { user_evaluation_score };
    let comm_state_clone = comm_state.inner().clone(); // 백그라운드 스레드로 move

    spawn(async move {
        println!("Background sync: Attempting to sync session end for {}", active_session_id);
        let _ = comm_state_clone.client.put(&url).bearer_auth(MOCK_AUTH_TOKEN).json(&request_body).send().await
            .map_err(|e| eprintln!("Background sync: Warning: Failed to sync session end for {}: {}", active_session_id, e));
        println!("Background sync: Session end sync attempt finished for {}.", active_session_id);
    });

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