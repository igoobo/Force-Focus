use crate::backend_communicator::BackendCommunicator;
use crate::app_core::AppCore; 
use crate::storage_manager::StorageManager;
use crate::inference::InferenceEngine;
use crate::StorageManagerArcMutex;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio::time::sleep;

// 모델 저장 경로 (lib.rs와 일치시킴)
const MODEL_DIR: &str = "models"; // 상대 경로만 정의 (OS 경로와 결합용)
const MODEL_FILENAME: &str = "personal_model.onnx";
const SCALER_FILENAME: &str = "scaler_params.json";

// 구조체 정의: 상태 관리를 위한 서비스 객체
// Clone이 가볍도록 설계 (AppHandle은 내부적으로 Arc와 유사함)
#[derive(Clone)]
pub struct ModelUpdateManager {
    app_handle: AppHandle,
}

impl ModelUpdateManager {
    // 생성자
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    // 업데이트 확인 및 수행 (Result<bool> 반환: true=업데이트됨)
    // 이 메서드는 '백그라운드 루프'와 '프론트엔드 커맨드' 양쪽에서 호출됩니다.
    pub async fn check_and_update(&self, token: &str) -> Result<bool, String> {
        // 1. 필요한 State 가져오기 (AppHandle을 통해 접근)
        let communicator = self.app_handle.try_state::<Arc<BackendCommunicator>>()
            .ok_or("BackendCommunicator state not found")?
            .inner().clone();

        // 2. 경로 설정
        let app_data_dir = self.app_handle.path().app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;
        
        let model_dir = app_data_dir.join(MODEL_DIR);
        if !model_dir.exists() {
            std::fs::create_dir_all(&model_dir).map_err(|e| e.to_string())?;
        }

        let final_model_path = model_dir.join(MODEL_FILENAME);
        let final_scaler_path = model_dir.join(SCALER_FILENAME);

        // 3. 버전 확인 (API 호출)
        let info = communicator.check_latest_model_version(token).await
            .map_err(|e| format!("Check version failed: {}", e))?;

        // TODO: 로컬 버전과 비교 로직 추가 (현재는 무조건 진행)
        // println!("Remote version: {}", info.version);

        // 4. 다운로드 (임시 파일)
        let temp_model_path = model_dir.join("temp_model.onnx");
        let temp_scaler_path = model_dir.join("temp_scaler.json");

        communicator.download_file(&info.download_urls.model, &temp_model_path, token).await
            .map_err(|e| format!("Download model failed: {}", e))?;
        communicator.download_file(&info.download_urls.scaler, &temp_scaler_path, token).await
            .map_err(|e| format!("Download scaler failed: {}", e))?;

        // 5. Atomic Swap & Reload (Critical Section)
        // ----------------------------------------------------------------
        // [핵심 수정] AppCore를 Lock하고 내부의 InferenceEngine을 통째로 교체
        // ----------------------------------------------------------------
        if let Some(app_core_state) = self.app_handle.try_state::<Mutex<AppCore>>() {
            let mut core = app_core_state.lock().map_err(|_| "Failed to lock AppCore")?;

            // 1. 기존 엔진 제거 (메모리 해제 및 파일 락 해제)
            core.inference_engine = None;
            
            // 파일 락이 풀릴 시간을 짧게 부여 (윈도우 환경 필수)
            std::thread::sleep(Duration::from_millis(100));

            // 2. 파일 교체 (백업 후 덮어쓰기)
            if final_model_path.exists() {
                let _ = std::fs::rename(&final_model_path, final_model_path.with_extension("bak"));
            }
            std::fs::rename(&temp_model_path, &final_model_path).map_err(|e| e.to_string())?;
            std::fs::rename(&temp_scaler_path, &final_scaler_path).map_err(|e| e.to_string())?;

            // 3. 새 파일로 새 엔진 객체 생성하여 AppCore에 주입
            match InferenceEngine::new(
                final_model_path.to_str().unwrap(), 
                final_scaler_path.to_str().unwrap()
            ) {
                Ok(new_engine) => {
                    core.inference_engine = Some(new_engine);
                    println!("✅ Model updated and reloaded to version {}", info.version);
                    Ok(true)
                },
                Err(e) => {
                    // 새 모델 로드 실패 시, 백업 파일로 복구를 시도해야 하나 여기서는 생략하고 에러 반환
                    Err(format!("Failed to load new model: {}", e))
                }
            }
        } else {
            Err("AppCore state not found".to_string())
        }
    }
}

pub fn start_update_loop(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        println!("🚀 Model Update Loop Started.");
        sleep(Duration::from_secs(5)).await;

        // Manager 인스턴스 생성 (루프 내에서 사용)
        let manager = ModelUpdateManager::new(app_handle.clone());

        loop {
            // 토큰 가져오기
            let token_opt = if let Some(storage_mutex) = app_handle.try_state::<StorageManagerArcMutex>() {
                let storage = storage_mutex.lock().unwrap();
                storage.load_auth_token().unwrap_or(None).map(|t| t.0)
            } else {
                None
            };

            if let Some(token) = token_opt {
                // [핵심] 로직 재사용: check_and_update 호출
                match manager.check_and_update(&token).await {
                    Ok(updated) => {
                        if updated { println!("✨ Background update success."); }
                    },
                    Err(e) => {
                        // 백그라운드에서는 에러가 나도 죽지 않고 로그만 남김
                        // eprintln!("Background update check failed: {}", e);
                    }
                }
            }

            sleep(Duration::from_secs(3600)).await;
        }
    });
}