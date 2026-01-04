use tauri::{AppHandle, Manager};
use std::time::Duration;
use tokio::time::sleep;
use std::sync::Arc;

use crate::StorageManagerArcMutex;
use crate::backend_communicator::BackendCommunicator;

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
    let storage_state = app.try_state::<StorageManagerArcMutex>()
        .ok_or("StorageManager state not found in AppHandle")?;

    // 2. 토큰 및 데이터 조회 (Mutex Lock 최소화)
    // - Lock을 잡고 데이터를 복사해온 뒤 즉시 해제하여 UI 블로킹 방지
    let (token, events) = {
        let storage = storage_state.lock().map_err(|e| e.to_string())?;

        // 로그인 여부 확인
        let token_data = storage.load_auth_token()?;
        let token = match token_data {
            Some((access, _, _)) => access,
            None => return Ok(()), // 토큰 없음 = 오프라인 모드 (조용히 리턴)
        };

        // 전송할 이벤트 조회 (최대 50개씩 끊어서 전송)
        let events = storage.get_unsynced_events(50)?;

        if events.is_empty() {
            return Ok(()); // 보낼 데이터 없음
        }

        (token, events)
    }; // 여기서 storage Lock 해제됨

    // 삭제를 위해 ID 목록 미리 백업
    let event_ids: Vec<i64> = events.iter().map(|e| e.id).collect();
    let count = event_ids.len();

    // 3. 백엔드 통신 모듈 가져오기
    // (BackendCommunicator는 내부적으로 Client를 가지고 있으며, 상태로 등록됨)
    let comm_state = app.try_state::<Arc<BackendCommunicator>>()
        .ok_or("BackendCommunicator state not found")?;

    // 4. 서버로 전송 (비동기)
    // (events 벡터의 소유권이 넘어감)
    // backend_communicator.rs에 구현된 sync_events_batch 메서드 호출
    comm_state.sync_events_batch(events, &token).await?;

    // 5. 전송 성공 시 로컬 데이터 삭제
    {
        let storage = storage_state.lock().map_err(|e| e.to_string())?;
        storage.delete_events_by_ids(&event_ids)?;
    }

    println!("Sync Manager: Successfully synced {} events to server.", count);

    Ok(())
}