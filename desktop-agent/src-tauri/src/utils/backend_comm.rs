// 파일 위치: src-tauri/src/backend_communicator.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{command, State, AppHandle};

// 백그라운드 동기화를 위해 tokio::spawn과 Arc, Mutex를 사용
use std::sync::{Arc, Mutex};
use tokio::spawn;

use dotenv::dotenv;
use std::env;

// 파일 I/O 및 스트림 처리
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use anyhow::Result;

// lib.rs에서 정의한 전역 상태 타입들
use crate::{
    ActiveSessionInfo, InputStatsArcMutex, SessionStateArcMutex, StorageManagerArcMutex, Task,
};

// StorageManager의 메서드를 호출하기 위해 모듈 import
use crate::managers::storage::{self, CachedEvent, LocalSchedule, LocalTask}; // LocalTask, LocalSchedule import
use crate::core::app::AppCore;

use std::time::{SystemTime, UNIX_EPOCH}; // 세션 시작 시간 생성용
use uuid::Uuid; // 로컬에서 임시 세션 ID 생성용

// --- 1. 상수 정의 ---

fn get_api_base_url() -> String {
    // 컴파일 타임에 환경 변수 'API_BASE_URL'을 읽어 바이너리에 박제
    // 배포 빌드 시 이 값이 고정 ($env:VITE_API_BASE_URL="http://YOUR_GCP_IP.nip.io:8000/api/v1"
    const BUILD_TIME_URL: Option<&str> = option_env!("API_BASE_URL");
    
    // 1순위: 빌드 시 주입된 URL (배포용)
    if let Some(url) = BUILD_TIME_URL {
        return url.to_string();
    }

    // 2순위: 런타임 .env 파일 (로컬 개발용)
    dotenv().ok();
    env::var("API_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000/api/v1".to_string())
}

// --- 2. API 요청/응답을 위한 구조체 ---

// 피드백 데이터 구조체 (서버 전송용)
#[derive(Debug, Serialize, Clone)]
pub struct FeedbackPayload {
    pub client_event_id: String,
    pub feedback_type: String,
    pub timestamp: String,
}

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
    pub client_event_id: String,
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
    pub start_date: Option<String>, // [New] "YYYY-MM-DD" (Optional)
    is_active: bool,
}

// ================================================================
// ML 모델 업데이트를 위한 DTO 및 메서드 확장
// ================================================================

#[derive(Debug, Deserialize, Clone)]
pub struct ModelVersionResponse {
    pub status: String,
    pub version: String,
    pub download_urls: ModelDownloadUrls,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelDownloadUrls {
    pub model: String,
    pub scaler: String,
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

    /// 최신 모델 버전 메타데이터 조회
    pub async fn check_latest_model_version(&self, token: &str) -> Result<ModelVersionResponse> {
        // 기존 스타일 준수: 헬퍼 함수로 URL 조합
        let url = format!("{}/desktop/models/latest", get_api_base_url());
        
        let resp = self.client.get(&url)
            .bearer_auth(token)
            .send()
            .await?;
            
        // 상태 코드 확인 (에러 시 anyhow::Error로 변환)
        let resp = resp.error_for_status()?;
        
        let info: ModelVersionResponse = resp.json().await?;
        Ok(info)
    }

    /// 범용 파일 다운로드 (모델 .onnx 및 스케일러 .json 공용)
    /// - endpoint: "/api/..." (상대경로) 또는 "https://..." (절대경로) 모두 처리
    pub async fn download_file(&self, endpoint: &str, save_path: &PathBuf, token: &str) -> Result<()> {
        // 유연한 URL 처리: endpoint가 http로 시작하면 그대로, 아니면 Base URL 결합
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!("{}{}", get_api_base_url(), endpoint)
        };
        
        println!("[BackendCommunicator] Downloading stream from: {}", url);

        let resp = self.client.get(&url)
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?;

        // 스트리밍 다운로드 구현 (메모리 효율적)
        let mut file = File::create(save_path).await?;
        let mut stream = resp.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?; // 네트워크 에러 전파
            file.write_all(&chunk).await?;
        }
        
        file.flush().await?;
        Ok(())
    }

    // 피드백 배치 전송
    pub async fn send_feedback_batch(
        &self,
        feedbacks: Vec<FeedbackPayload>,
        token: &str,
    ) -> Result<(), String> {
        let url = format!("{}/desktop/feedback/batch", get_api_base_url());

        if feedbacks.is_empty() { return Ok(()); }

        println!("Sending {} feedback items to {}", feedbacks.len(), url);

        let response = self.client.post(&url)
            .bearer_auth(token)
            .json(&feedbacks)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            println!("✅ Feedback batch sent successfully.");
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(format!("Server returned error {}: {}", status, text))
        }
    }

    // 최신 모델 다운로드 (스트리밍)
    pub async fn download_latest_model(
        &self,
        save_path: PathBuf,
        token: &str,
    ) -> Result<(), String> {
        let url = format!("{}/desktop/models/latest", get_api_base_url());
        println!("Downloading model from {}", url);

        let response = self.client.get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Download failed: {}", response.status()));
        }

        let mut file = File::create(&save_path).await
            .map_err(|e| format!("File create error: {}", e))?;

        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|e| format!("Chunk error: {}", e))?;
            file.write_all(&chunk).await
                .map_err(|e| format!("Write error: {}", e))?;
        }

        file.flush().await.map_err(|e| format!("Flush error: {}", e))?;
        println!("✅ Model downloaded to {:?}", save_path);
        Ok(())
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
                        client_event_id: e.client_event_id,
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
        let url = format!("{}/desktop/data/tasks", get_api_base_url());

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
        let url = format!("{}/desktop/data/schedules", get_api_base_url());

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
                    start_date: s.start_date, // [Mapped]
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

/// '개입'에 대한 사용자 피드백을 서버로 전송하고, 즉시 로컬 상태를 리셋하는 커맨드
#[command]
pub async fn submit_feedback(
    feedback_type: String,
    // 통신 객체 및 세션 상태
    comm_state: State<'_, Arc<BackendCommunicator>>,
    session_state_mutex: State<'_, SessionStateArcMutex>,
    // LSN 저장용
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
    // FSM 리셋용 AppCore 상태 추가
    app_core_state: State<'_, Mutex<AppCore>>, 
) -> Result<(), String> {
    
    // 1. 현재 모니터링 중인 이벤트 ID 조회 (AppCore에서 가져옴)
    let client_event_id = {
        let app = app_core_state.lock().map_err(|_| "Failed to lock AppCore")?;
        // 만약 모니터링 시작 전이라 ID가 없다면, 새로 생성하거나 에러 처리
        app.current_event_id.clone().unwrap_or_else(|| format!("evt-fallback-{}", Uuid::new_v4()))
    };
    

    // 세션 ID를 가장 먼저 조회 (LSN 저장과 백그라운드 전송 모두에 사용하기 위함)
    let session_id = {
        let session_state = session_state_mutex.lock().map_err(|e| e.to_string())?;
        session_state.as_ref()
            .map(|s| s.session_id.clone())
            .unwrap_or_else(|| "unknown-session".to_string()) // 세션이 없을 때의 처리
    };

    

    // 2. LSN(로컬 DB)에 저장 (기존 로직 유지)
    {
        let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;
        storage_manager.cache_feedback(&client_event_id, &feedback_type)?;
        println!("Feedback cached to LSN successfully.");
    }

    // 3. FSM 즉시 리셋 (오버레이 해제)
    // 사용자 경험(UX)을 위해 UI를 즉시 평화 상태로 복구
    {
        // lock 범위를 최소화하기 위해 별도 블록 사용
        let mut app = app_core_state.lock().map_err(|_| "Failed to lock AppCore")?;
        
        if feedback_type == "is_work" {
            // 현재 활성 창 이름 가져오기 (AppCore 내부 메서드 활용 권장)
            // 임시로 "Unknown" 처리하거나, commands.rs의 헬퍼 함수 활용 가능
            // 여기서는 단순 리셋에 집중
            app.state_engine.manual_reset();
            println!("🔄 FSM State Reset by User Feedback");
            
            // (선택 사항) Local Cache 업데이트 로직도 여기에 추가 가능
            // app.inference_engine.update_local_cache(...);
        } else {
            // "distraction_ignored" 등 다른 피드백일 경우에도
            // 일단 오버레이는 꺼주는 게 UX상 좋음 (또는 유지 정책에 따라 결정)
            app.state_engine.manual_reset();
        }
    }

    // 4. 백그라운드 전송 (Communicator 활용)
    // UI 스레드를 차단하지 않기 위해 spawn 사용
    let comm = comm_state.inner().clone();
    let feedback_type_clone = feedback_type.clone();
    let client_event_id_clone = client_event_id.clone();

    // 토큰 조회
    let token = {
        let storage = storage_manager_mutex.lock().map_err(|e| e.to_string())?;
        storage.load_auth_token().unwrap_or(None).map(|t| t.0)
    };

    if let Some(auth_token) = token {
        spawn(async move {
            let payload = FeedbackPayload {
                client_event_id: client_event_id_clone,
                feedback_type: feedback_type_clone,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            
            // 공용 메서드 호출
            if let Err(e) = comm.send_feedback_batch(vec![payload], &auth_token).await {
                eprintln!("Background Feedback Sync Failed: {}", e);
            } else {
                println!("Background Feedback Sync Success");
            }
        });
    }

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
    app_handle: AppHandle,
    user_evaluation_score: u8,
    // comm_state도 백그라운드 동기화를 위해 Arc<BackendCommunicator>를 받도록 변경
    comm_state: State<'_, Arc<BackendCommunicator>>,
    session_state_mutex: State<'_, SessionStateArcMutex>,
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
    app_core_state: State<'_, Mutex<AppCore>>, // [Fix] FSM 리셋을 위해 추가
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

        // [Fix] 세션 종료 시 오버레이 숨기기 및 FSM 리셋 (UX: 즉시 반응)
        // hide_overlay는 내부적으로 AppCore.manual_reset()을 호출합니다.
        use crate::commands::window;
        if let Err(e) = crate::commands::window::hide_overlay(app_handle.clone(), app_core_state) {
            eprintln!("Warning: Failed to hide overlay on session end: {}", e);
        }

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
