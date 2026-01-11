// 파일 위치: src-tauri/src/backend_communicator.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{command, State};

// 백그라운드 동기화를 위해 tokio::spawn과 Arc, Mutex를 사용
use std::sync::{Arc, Mutex};
use tokio::spawn;

use dotenv::dotenv;
use std::env;

// lib.rs에서 정의한 전역 상태 타입들
use crate::{
    ActiveSessionInfo, InputStatsArcMutex, SessionStateArcMutex, StorageManagerArcMutex, Task,
};

// StorageManager의 메서드를 호출하기 위해 모듈 import
use crate::storage_manager::{self, CachedEvent, LocalSchedule, LocalTask}; // LocalTask, LocalSchedule import

use std::time::{SystemTime, UNIX_EPOCH}; // 세션 시작 시간 생성용
use uuid::Uuid; // 로컬에서 임시 세션 ID 생성용

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
    event_id: &'a str,      // 이벤트 ID (임시)
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

// 이벤트 배치 전송 요청 모델 (백엔드 스키마와 일치)
#[derive(Debug, Serialize)]
struct EventBatchRequest {
    events: Vec<EventData>,
}

#[derive(Debug, Serialize)]
struct EventData {
    session_id: String,
    timestamp: i64,
    app_name: String,
    window_title: String,
    activity_vector: serde_json::Value,
}

// 백엔드 Task API 응답 모델 (Schema: TaskRead)
#[derive(Debug, Deserialize)]
struct ApiTask {
    id: String,
    user_id: String,
    name: String,
    description: Option<String>,
    status: String,
    target_executable: Option<String>,
    target_arguments: Option<String>,
    // created_at, due_date 등은 필요 시 추가
}

// 백엔드 Schedule API 응답 모델 (Schema: ScheduleRead)
#[derive(Debug, Deserialize)]
struct ApiSchedule {
    id: String,
    user_id: String,
    task_id: Option<String>,
    name: String,
    start_time: String, // "HH:MM:SS" (Time 객체는 문자열로 옴)
    end_time: String,
    days_of_week: Vec<u8>,
    is_active: bool,
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

    // [핵심 추가] sync_manager가 호출할 동기화 메서드
    pub async fn sync_events_batch(
        &self,
        events: Vec<CachedEvent>,
        token: &str,
    ) -> Result<(), String> {
        let url = format!("{}/events/batch", get_api_base_url());

        // CachedEvent -> EventData 변환
        let event_data_list: Vec<EventData> = events
            .into_iter()
            .filter_map(|e| {
                // LSN에 저장된 JSON 문자열을 serde_json::Value 객체로 파싱
                match serde_json::from_str(&e.activity_vector) {
                    Ok(json_val) => Some(EventData {
                        session_id: e.session_id,
                        timestamp: e.timestamp,
                        app_name: e.app_name,
                        window_title: e.window_title,
                        activity_vector: json_val,
                    }),
                    Err(err) => {
                        eprintln!(
                            "Failed to parse activity_vector JSON for event {}: {}",
                            e.id, err
                        );
                        None // 파싱 실패한 데이터는 스킵 (또는 별도 처리)
                    }
                }
            })
            .collect();

        if event_data_list.is_empty() {
            return Ok(());
        }

        let request_body = EventBatchRequest {
            events: event_data_list,
        };

        println!("Syncing {} events to {}", request_body.events.len(), url);

        let response = self
            .client
            .post(&url)
            .bearer_auth(token)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            println!("Sync success!");
            Ok(())
        } else {
            // 서버 에러 메시지 확인
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(format!("Server returned error {}: {}", status, text))
        }
    }

    // ---  데이터 다운로드 (Fetch Only) ---
    // StorageManager 의존성을 제거하고 데이터를 반환

    /// 서버에서 Task 목록을 받아옴 (저장은 호출자가 수행)
    pub async fn fetch_tasks(&self, token: &str) -> Result<Vec<LocalTask>, String> {
        let url = format!("{}/tasks", get_api_base_url());

        let response = self
            .client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch tasks: {}", e))?;

        if response.status().is_success() {
            let api_tasks: Vec<ApiTask> = response
                .json()
                .await
                .map_err(|e| format!("JSON parse error: {}", e))?;

            let local_tasks: Vec<LocalTask> = api_tasks
                .into_iter()
                .map(|t| LocalTask {
                    id: t.id,
                    user_id: t.user_id,
                    task_name: t.name,
                    description: t.description,
                    target_executable: t.target_executable,
                    target_arguments: t.target_arguments,
                    status: t.status,
                })
                .collect();

            Ok(local_tasks)
        } else {
            Err(format!("Server error (Tasks): {}", response.status()))
        }
    }

    /// 서버에서 Schedule 목록을 받아옴 (저장은 호출자가 수행)
    pub async fn fetch_schedules(&self, token: &str) -> Result<Vec<LocalSchedule>, String> {
        let url = format!("{}/schedules", get_api_base_url());

        let response = self
            .client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch schedules: {}", e))?;

        if response.status().is_success() {
            let api_schedules: Vec<ApiSchedule> = response
                .json()
                .await
                .map_err(|e| format!("JSON parse error: {}", e))?;

            let local_schedules: Vec<LocalSchedule> = api_schedules
                .into_iter()
                .map(|s| LocalSchedule {
                    id: s.id,
                    user_id: s.user_id,
                    task_id: s.task_id,
                    name: s.name,
                    start_time: s.start_time,
                    end_time: s.end_time,
                    days_of_week: s.days_of_week,
                    is_active: s.is_active,
                })
                .collect();

            Ok(local_schedules)
        } else {
            Err(format!("Server error (Schedules): {}", response.status()))
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
    user_id: String,
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
) -> Result<(), String> {
    let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;

    // LSN에 토큰 저장
    storage_manager.save_auth_token(&access_token, &refresh_token, &user_email, &user_id)?;

    println!("User logged in: {}", user_email);
    Ok(())
}

// --- 로그아웃 커맨드 ---
#[command]
pub fn logout(storage_manager_mutex: State<'_, StorageManagerArcMutex>) -> Result<(), String> {
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
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
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
) -> Result<ActiveSessionInfo, String> {
    // React에 ActiveSessionInfo 반환

    // 1. '쓰기' 락 & 토큰 로드: .await 전에 LSN과 전역 상태 즉시 업데이트
    // 반환값: (ActiveSessionInfo, Option<String>) -> (세션정보, 토큰)
    let (info, auth_token) = {
        // 락 범위를 제한하기 위해 새 스코프 생성
        let mut session_state = session_state_mutex
            .lock()
            .map_err(|e| format!("State lock error: {}", e))?;
        let storage_manager = storage_manager_mutex
            .lock()
            .map_err(|e| format!("Storage lock error: {}", e))?;

        let mut input_stats = input_stats_mutex
            .lock()
            .map_err(|e| format!("InputStats lock error: {}", e))?;

        if session_state.is_some() {
            return Err("Session already active.".to_string());
        }

        // LSN에서 인증 토큰 로드 (로그인 상태 확인)
        // load_auth_token은 Result<Option<(access, refresh, email)>, String>을 반환한다고 가정
        let token = storage_manager
            .load_auth_token()
            .unwrap_or(None) // 에러나면 무시 (오프라인/미로그인으로 간주)
            .map(|t| t.0); // (access_token, refresh_token, email, user_id) 중 access_token만 추출

        // 서버 응답을 기다리지 않고, 로컬에서 즉시 세션 정보를 생성
        let session_id = format!("local-{}", Uuid::new_v4());
        let start_time_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

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

            println!(
                "Background sync: Attempting to sync session {} to server...",
                info_clone.session_id
            );

            // MOCK_AUTH_TOKEN 대신 진짜 token 사용
            match comm_state_clone
                .client
                .post(&url)
                .bearer_auth(token)
                .json(&request_body)
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    // 성공 시 서버 응답(Server Session ID) 처리 로직이 필요할 수 있음
                    println!(
                        "Background sync: Session {} synced successfully.",
                        info_clone.session_id
                    );
                }
                _ => {
                    eprintln!(
                        "Background sync: Failed to sync session {}. Will retry later.",
                        info_clone.session_id
                    );
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

        let active_session_id = session_state
            .as_ref()
            .map(|s| s.session_id.clone())
            .ok_or_else(|| "No active session to end.".to_string())?;

        // 백그라운드 동기화를 위해 LSN에서 토큰 읽기
        let token = storage_manager
            .load_auth_token()
            .unwrap_or(None)
            .map(|t| t.0);

        // LSN 및 전역 상태 정리 (먼저 실행)
        storage_manager.delete_active_session()?;
        *session_state = None; // 전역 상태 초기화

        println!(
            "Session ID {} successfully ended locally (score: {}).",
            active_session_id, user_evaluation_score
        );
        (active_session_id, token) // 락 해제 전 데이터 복사
                                   // MutexGuard('session_state', 'storage_manager')는 여기서 자동으로 drop (락 해제)
    };

    // 2. 백그라운드 동기화: UI를 기다리게 하지 않음
    if let Some(token) = auth_token {
        //  환경 변수 URL 사용
        let url = format!("{}/sessions/{}", get_api_base_url(), active_session_id);

        let request_body = SessionEndRequest {
            user_evaluation_score,
        };
        let comm_state_clone = comm_state.inner().clone();

        spawn(async move {
            println!(
                "Background sync: Attempting to sync session end for {}",
                active_session_id
            );

            // MOCK_AUTH_TOKEN 대신 진짜 token 사용
            let _ = comm_state_clone
                .client
                .put(&url)
                .bearer_auth(token)
                .json(&request_body)
                .send()
                .await
                .map_err(|e| {
                    eprintln!(
                        "Background sync: Warning: Failed to sync session end for {}: {}",
                        active_session_id, e
                    )
                });

            println!(
                "Background sync: Session end sync attempt finished for {}.",
                active_session_id
            );
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
    session_state_mutex: State<'_, SessionStateArcMutex>,
) -> Result<Option<ActiveSessionInfo>, String> {
    let session_state = session_state_mutex
        .lock()
        .map_err(|e| format!("State lock error: {}", e))?;

    // [!] 전역 상태('SessionState')를 복제(clone)하여 React로 반환
    Ok(session_state.clone())
}

//  앱 시작 시 로그인 상태 확인 (Auto-Login)
#[command]
pub fn check_auth_status(
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
) -> Result<Option<String>, String> {
    let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;

    // LSN에서 토큰 로드 (Access, Refresh, Email)
    let token_data = storage_manager
        .load_auth_token()
        .map_err(|e| e.to_string())?;

    // 토큰이 있으면 이메일 반환, 없으면 None
    if let Some((_, _, email, _)) = token_data {
        println!("Auto-login: Found valid token for {}", email);
        Ok(Some(email))
    } else {
        Ok(None)
    }
}

//  Task / LSN 데이터 연동
#[command]
pub fn get_tasks(
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
) -> Result<Vec<Task>, String> {
    let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;

    // 1. 현재 로그인한 사용자 ID 확인 (격리)
    let user_id = match storage_manager
        .load_auth_token()
        .map_err(|e| e.to_string())?
    {
        Some((_, _, _, uid)) => uid,
        None => return Ok(vec![]), // 로그인 안 했으면 빈 목록 반환 (오프라인/게스트 정책에 따라 다름)
    };

    // 2. LSN에서 해당 유저의 Task 조회
    let local_tasks = storage_manager
        .get_tasks_by_user(&user_id)
        .map_err(|e| e.to_string())?;

    println!(
        "get_tasks: Found {} tasks for user {}",
        local_tasks.len(),
        user_id
    );

    // 3. LocalTask -> Task (프론트엔드용) 변환
    let tasks: Vec<Task> = local_tasks
        .into_iter()
        .map(|t| Task {
            id: t.id,
            user_id: t.user_id,
            task_name: t.task_name,
            description: t.description.unwrap_or_default(),
            // DB에는 날짜 필드가 없으므로 일단 빈 값 처리 (추후 필요하면 DB 마이그레이션)
            due_date: "".to_string(),
            status: t.status,
            target_executable: t.target_executable.unwrap_or_default(),
            // "arg1 arg2" (String) -> ["arg1", "arg2"] (Vec<String>)
            target_arguments: t
                .target_arguments
                .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            created_at: "".to_string(),
            updated_at: "".to_string(),
        })
        .collect();

    Ok(tasks)
}
