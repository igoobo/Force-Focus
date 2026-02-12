use crate::backend_communicator::BackendCommunicator;
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

                let model_dir = app_data_dir.join(MODEL_DIR);
                if !model_dir.exists() {
                    let _ = std::fs::create_dir_all(&model_dir);
                }

                let final_model_path = model_dir.join(MODEL_FILENAME);
                let final_scaler_path = model_dir.join(SCALER_FILENAME);

                // ================================================================
                // 새로운 업데이트 파이프라인 (Version Check -> Download -> Swap)
                // ================================================================

                // 3. 모델 다운로드 시도 (Communicator 로직 재사용)

                // Step A: 버전 확인
                match communicator.check_latest_model_version(&token).await {
                    Ok(info) => {
                        // TODO: 현재 로컬 버전과 비교하는 로직 추가 가능 (storage_manager에 저장된 버전 등)
                        // 여기서는 일단 무조건 업데이트 시도한다고 가정 (또는 info.version 비교)

                        println!("✨ New version found: {}", info.version);

                        // Step B: 임시 파일로 다운로드 (Atomic Update 준비)
                        let temp_model_path = model_dir.join("temp_model.onnx");
                        let temp_scaler_path = model_dir.join("temp_scaler.json");

                        let download_result = async {
                            communicator.download_file(&info.download_urls.model, &temp_model_path, &token).await?;
                            communicator.download_file(&info.download_urls.scaler, &temp_scaler_path, &token).await?;
                            Ok::<(), anyhow::Error>(())
                        }.await;

                        match download_result {
                            Ok(_) => {
                                // Step C: 파일 교체 및 엔진 리로드 (Critical Section)
                                if let Some(engine_state) = app_handle.try_state::<Mutex<InferenceEngine>>() {
                                    match engine_state.lock() {
                                        Ok(mut engine) => {
                                            // 1. Unload (Windows File Lock 해제)
                                            engine.unload_model();
                                            
                                            // 2. 파일 교체 (Rename)
                                            // 백업 (선택사항)
                                            if final_model_path.exists() {
                                                let _ = std::fs::rename(&final_model_path, final_model_path.with_extension("bak"));
                                            }
                                            
                                            // 덮어쓰기
                                            if let Err(e) = std::fs::rename(&temp_model_path, &final_model_path) {
                                                eprintln!("🔥 File Swap Failed (Model): {}", e);
                                            }
                                            if let Err(e) = std::fs::rename(&temp_scaler_path, &final_scaler_path) {
                                                eprintln!("🔥 File Swap Failed (Scaler): {}", e);
                                            }

                                            // 3. Reload
                                            // 잠시 대기 (OS 파일 핸들 완전 해제 보장)
                                            // 비동기 컨텍스트지만 Mutex 안이라 thread::sleep 사용 (주의)
                                            // 짧은 시간이므로 허용
                                            std::thread::sleep(Duration::from_millis(100)); 
                                            
                                            match engine.load_model(&final_model_path) {
                                                Ok(_) => println!("✅ Hot-Swap Complete: Version {}", info.version),
                                                Err(e) => eprintln!("🔥 Reload Failed: {}", e),
                                            }
                                        }
                                        Err(e) => eprintln!("Failed to lock engine: {}", e),
                                    }
                                }
                            }
                            Err(e) => eprintln!("Download failed: {}", e),
                        }
                    }
                    Err(e) => {
                        // 버전 확인 실패 (네트워크 오류 or 최신 버전 없음 등)
                        // 조용히 넘어감
                        // eprintln!("Update check failed: {}", e); 
                    }
                }
            }

            // 4. 다음 주기 대기 (1시간)
            sleep(Duration::from_secs(3600)).await;
        }
    });
}