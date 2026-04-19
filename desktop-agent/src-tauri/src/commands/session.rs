// 파일 위치: src-tauri/src/commands/session.rs
// backend_comm.rs에서 분리된 세션 관련 Tauri 커맨드 (U-3 해결)

use tauri::{command, State, AppHandle};
use std::sync::{Arc, Mutex};
use tokio::spawn;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::{
    ActiveSessionInfo, InputStatsArcMutex, SessionStateArcMutex, StorageManagerArcMutex,
};
use crate::utils::api::{
    BackendCommunicator, FeedbackPayload, SessionStartRequest, SessionEndRequest,
    get_api_base_url,
};
use crate::core::app::AppCore;

/// '개입'에 대한 사용자 피드백을 서버로 전송하고, 즉시 로컬 상태를 리셋하는 커맨드
#[command]
pub async fn submit_feedback(
    feedback_type: String,
    comm_state: State<'_, Arc<BackendCommunicator>>,
    session_state_mutex: State<'_, SessionStateArcMutex>,
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
    app_core_state: State<'_, Mutex<AppCore>>, 
) -> Result<(), String> {
    
    let client_event_id = {
        let app = app_core_state.lock().map_err(|_| "Failed to lock AppCore")?;
        app.current_event_id.clone().unwrap_or_else(|| format!("evt-fallback-{}", Uuid::new_v4()))
    };
    
    let _session_id = {
        let session_state = session_state_mutex.lock().map_err(|e| e.to_string())?;
        session_state.as_ref()
            .map(|s| s.session_id.clone())
            .unwrap_or_else(|| "unknown-session".to_string())
    };

    // LSN(로컬 DB)에 저장
    {
        let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;
        storage_manager.cache_feedback(&client_event_id, &feedback_type)?;
        println!("Feedback cached to LSN successfully.");
    }

    // FSM 즉시 리셋 (오버레이 해제)
    {
        let mut app = app_core_state.lock().map_err(|_| "Failed to lock AppCore")?;
        
        if feedback_type == "is_work" {
            app.state_engine.manual_reset();
            println!("🔄 FSM State Reset by User Feedback");
        } else {
            app.state_engine.manual_reset();
        }
    }

    // 백그라운드 전송
    let comm = comm_state.inner().clone();
    let feedback_type_clone = feedback_type.clone();
    let client_event_id_clone = client_event_id.clone();

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
            
            if let Err(e) = comm.send_feedback_batch(vec![payload], &auth_token).await {
                eprintln!("Background Feedback Sync Failed: {}", e);
            } else {
                println!("Background Feedback Sync Success");
            }
        });
    }

    Ok(())
}

/// 세션 시작 커맨드
#[command]
pub async fn start_session(
    task_id: Option<String>,
    goal_duration: u32,
    comm_state: State<'_, Arc<BackendCommunicator>>,
    session_state_mutex: State<'_, SessionStateArcMutex>,
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
    input_stats_mutex: State<'_, InputStatsArcMutex>,
    app_core_state: State<'_, Mutex<AppCore>>,
) -> Result<ActiveSessionInfo, String> {
    let (info, auth_token) = {
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

        let token = storage_manager
            .load_auth_token()
            .unwrap_or(None)
            .map(|t| t.0);

        let session_id = format!("local-{}", Uuid::new_v4());
        let start_time_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        let info = ActiveSessionInfo {
            session_id: session_id.clone(),
            task_id: task_id.clone(),
            start_time_s,
        };

        storage_manager.save_active_session(&info)?;
        *session_state = Some(info.clone());

        input_stats.meaningful_input_events = 0;
        input_stats.last_meaningful_input_timestamp_ms = start_time_s * 1000;
        input_stats.last_mouse_move_timestamp_ms = start_time_s * 1000;

        eprintln!("Session started (Offline-First). ID: {}", info.session_id);

        if let Ok(mut app_core) = app_core_state.lock() {
            app_core.state_engine.manual_reset();
            app_core.last_inference_result = crate::ai::inference::InferenceResult::Inlier;
        }

        (info, token)
    };

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

            match comm_state_clone
                .client
                .post(&url)
                .bearer_auth(token)
                .json(&request_body)
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    println!("Background sync: Session synced successfully.");
                }
                _ => {
                    eprintln!("Background sync: Failed to sync session. Will retry later.");
                }
            }
        });
    }

    Ok(info)
}

/// 세션 종료 커맨드
#[command]
pub async fn end_session(
    app_handle: AppHandle,
    user_evaluation_score: u8,
    comm_state: State<'_, Arc<BackendCommunicator>>,
    session_state_mutex: State<'_, SessionStateArcMutex>,
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
    app_core_state: State<'_, Mutex<AppCore>>,
) -> Result<(), String> {
    let (active_session_id, auth_token) = {
        let mut session_state = session_state_mutex.lock().map_err(|e| e.to_string())?;
        let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;

        let active_session_id = session_state
            .as_ref()
            .map(|s| s.session_id.clone())
            .ok_or_else(|| "No active session to end.".to_string())?;

        let token = storage_manager
            .load_auth_token()
            .unwrap_or(None)
            .map(|t| t.0);

        storage_manager.delete_active_session()?;
        *session_state = None;

        if let Ok(mut app_core) = app_core_state.lock() {
            app_core.state_engine.manual_reset();
            app_core.last_inference_result = crate::ai::inference::InferenceResult::Inlier;
        }

        if let Err(e) = crate::commands::window::hide_overlay(app_handle.clone(), app_core_state) {
            eprintln!("Warning: Failed to hide overlay on session end: {}", e);
        }

        println!(
            "Session ID {} successfully ended locally (score: {}).",
            active_session_id, user_evaluation_score
        );
        (active_session_id, token)
    };

    if let Some(token) = auth_token {
        let url = format!("{}/sessions/{}", get_api_base_url(), active_session_id);
        let request_body = SessionEndRequest { user_evaluation_score };
        let comm_state_clone = comm_state.inner().clone();

        spawn(async move {
            let _ = comm_state_clone
                .client
                .put(&url)
                .bearer_auth(token)
                .json(&request_body)
                .send()
                .await;
        });
    }

    Ok(())
}

/// 타이머 위젯 동기화를 위한 PULL API
#[command]
pub fn get_current_session_info(
    session_state_mutex: State<'_, SessionStateArcMutex>,
) -> Result<Option<ActiveSessionInfo>, String> {
    let session_state = session_state_mutex
        .lock()
        .map_err(|e| format!("State lock error: {}", e))?;
    Ok(session_state.clone())
}
