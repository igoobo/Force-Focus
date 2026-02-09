use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio::time::sleep;
use serde_json::json;
use chrono::{DateTime, Utc}; // 날짜 변환용

use crate::backend_communicator::{BackendCommunicator, FeedbackPayload};
use crate::StorageManagerArcMutex;

/// 백그라운드 동기화 루프 시작
pub fn start_sync_loop(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        println!("Sync Manager: Started background sync loop (Interval: 60s)");

        loop {
            // 1분 대기 (60초)
            sleep(Duration::from_secs(60)).await;

            // 동기화 작업 실행 (실패해도 로그만 남기고 루프 유지)
            if let Err(e) = process_sync(&app_handle).await {
                eprintln!("Sync Manager Error: {}", e);
            }
        }
    });
}

/// 실제 동기화 로직 (1회 실행)
async fn process_sync(app: &AppHandle) -> Result<(), String> {
    // 1. LSN 상태 가져오기
    let storage_state = app
        .try_state::<StorageManagerArcMutex>()
        .ok_or("StorageManager state not found in AppHandle")?;

    // 백엔드 통신 모듈 가져오기
    // (BackendCommunicator는 내부적으로 Client를 가지고 있으며, 상태로 등록됨)
    // Arc<BackendCommunicator> 타입으로 가져오기
    let comm_state = app
        .try_state::<Arc<BackendCommunicator>>()
        .ok_or("BackendCommunicator state not found")?;

    // 2. 토큰 확인 (로그인 여부)
    // 이벤트를 조회하기 전에 토큰부터 확인
    let token = {
        let storage = storage_state.lock().map_err(|e| e.to_string())?;
        match storage.load_auth_token()? {
            Some((access, _, _, _)) => access,
            None => return Ok(()), // 토큰 없음 = 오프라인 모드 (동기화 전체 스킵)
        }
    }; // 여기서 storage Lock 해제

    // --- [A] Down-Sync: 서버 데이터 가져오기 (스케줄 & 태스크) ---
    // 2-1. Task 다운로드
    let fetched_tasks = match comm_state.fetch_tasks(&token).await {
        Ok(t) => Some(t),
        Err(e) => {
            eprintln!("Sync Manager: Failed to fetch tasks: {}", e);
            None
        }
    };

    // 2-2. Schedule 다운로드
    let fetched_schedules = match comm_state.fetch_schedules(&token).await {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("Sync Manager: Failed to fetch schedules: {}", e);
            None
        }
    };

    // 3. 로컬 DB 저장 (Lock 필요)
    {
        let storage = storage_state.lock().map_err(|e| e.to_string())?;

        if let Some(tasks) = fetched_tasks {
            if let Err(e) = storage.sync_tasks(tasks) {
                eprintln!("Sync Manager: Failed to sync tasks to DB: {}", e);
            }
        }

        if let Some(schedules) = fetched_schedules {
            if let Err(e) = storage.sync_schedules(schedules) {
                eprintln!("Sync Manager: Failed to sync schedules to DB: {}", e);
            }
        }
    } // lock 해제

    // --- [B] Up-Sync: 로컬 데이터 올리기 (이벤트) ---
    // 4. 전송할 데이터 조회 (Lock)
    let events = {
        let storage = storage_state.lock().map_err(|e| e.to_string())?;
        storage.get_unsynced_events(50)?
    };

    if !events.is_empty() {
        let event_ids: Vec<i64> = events.iter().map(|e| e.id).collect();
        let count = event_ids.len();

        // 5. 서버 전송 (Async, No Lock)
        comm_state.sync_events_batch(events, &token).await?;

        // 6. 전송 성공 시 삭제 (Lock)
        {
            let storage = storage_state.lock().map_err(|e| e.to_string())?;
            storage.delete_events_by_ids(&event_ids)?;
        }
        println!("Sync Manager: Successfully uploaded {} events.", count);
    }

    // --- [C] Up-Sync: 사용자 피드백  ---

    // 1. 미전송 데이터 조회 (최대 50개)
    let feedbacks = {
        let storage = storage_state.lock().map_err(|e| e.to_string())?;
        storage.get_unsynced_feedbacks(50)?
    };

    if !feedbacks.is_empty() {
        let feedback_ids: Vec<i64> = feedbacks.iter().map(|f| f.id).collect();
        let count = feedbacks.len();

        // 2. [매핑] DB 구조체 -> API Payload 변환
        // event_id는 서버 스키마에 없으므로 context_snapshot JSON에 넣어줍니다.
        let payloads: Vec<FeedbackPayload> = feedbacks.into_iter().map(|f| {
             let dt = DateTime::<Utc>::from_timestamp(f.timestamp, 0).unwrap_or(Utc::now());
             
             FeedbackPayload {
                client_event_id: f.event_id,
                feedback_type: f.feedback_type,
                timestamp: dt.to_rfc3339(),
            }
        }).collect();

        // 3. 서버 전송
        comm_state.send_feedback_batch(payloads, &token).await?;

        // 4. 전송 성공 시 로컬 삭제 (Transactional Delete)
        {
            let storage = storage_state.lock().map_err(|e| e.to_string())?;
            storage.delete_feedbacks_by_ids(&feedback_ids)?;
        }
        
        println!("✅ Sync Manager: Uploaded and cleaned up {} feedbacks.", count);
    }

    Ok(())
}
