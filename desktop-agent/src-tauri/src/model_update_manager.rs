use crate::backend_communicator::BackendCommunicator;
use crate::storage_manager::StorageManager;
use crate::StorageManagerArcMutex;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio::time::sleep;

// 모델 저장 경로 (lib.rs와 일치시킴)
const MODEL_DIR: &str = "resources/models";
const MODEL_FILENAME: &str = "personal_model.onnx";

pub fn start_update_loop(app_handle: AppHandle) {
    // 백그라운드 스레드(Green Thread) 시작
    tauri::async_runtime::spawn(async move {
        println!("🚀 Model Update Manager Started.");
        
        // 앱 시작 직후 5초 대기 (네트워크 안정화 및 로그인 처리 대기)
        sleep(Duration::from_secs(5)).await;

        loop {
            // 1. 상태 객체 가져오기
            // Communicator는 lib.rs에서 Arc<BackendCommunicator>로 등록됨
            let communicator = match app_handle.try_state::<Arc<BackendCommunicator>>() {
                Some(state) => state.inner().clone(),
                None => {
                    eprintln!("ModelManager: BackendCommunicator state not found.");
                    sleep(Duration::from_secs(10)).await;
                    continue;
                }
            };

            let storage_manager_mutex = match app_handle.try_state::<StorageManagerArcMutex>() {
                Some(state) => state.inner().clone(),
                None => {
                    eprintln!("ModelManager: StorageManager state not found.");
                    sleep(Duration::from_secs(10)).await;
                    continue;
                }
            };

            // 2. 인증 토큰 확인
            let token_opt = {
                let storage = storage_manager_mutex.lock().unwrap(); // 간단한 락
                storage.load_auth_token().unwrap_or(None).map(|t| t.0)
            };

            if let Some(token) = token_opt {
                println!("🤖 Checking for model updates...");

                // OS 표준 데이터 경로 사용 (AppData)
                // app_handle.path().app_data_dir()은 Result를 반환하므로 처리 필요
                let app_data_dir = match app_handle.path().app_data_dir() {
                    Ok(dir) => dir,
                    Err(e) => {
                        eprintln!("Failed to get app data dir: {}", e);
                        sleep(Duration::from_secs(3600)).await;
                        continue;
                    }
                };

                // 저장 경로: AppData/Roaming/com.force.focus/models/personal_model.onnx
                let mut save_path = app_data_dir.clone();
                save_path.push("models"); // 하위 폴더
                
                // 폴더가 없으면 생성
                if !save_path.exists() {
                    let _ = std::fs::create_dir_all(&save_path);
                }
                
                save_path.push(MODEL_FILENAME);

                // 3. 모델 다운로드 시도 (Communicator 로직 재사용)
                match communicator.download_latest_model(save_path.clone(), &token).await {
                    Ok(_) => println!("✅ Model update check completed."),
                    Err(e) => eprintln!("⚠️ Model update failed: {}", e),
                }
            } else {
                // 로그인이 안 되어 있으면 조용히 대기
                // println!("ModelManager: User not logged in. Skipping update.");
            }

            // 4. 다음 주기 대기 (예: 1시간 = 3600초)
            // 개발 중 테스트를 위해 5분(300초) 등으로 짧게 잡아도 됩니다.
            sleep(Duration::from_secs(3600)).await;
        }
    });
}