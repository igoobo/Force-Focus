// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// --- 1. 모듈 선언 ---
pub mod ai;
pub mod commands;
pub mod core;
pub mod managers;
pub mod utils;

// --- 2. 전역 use ---
use crate::ai::inference::InferenceEngine;
use crate::ai::model_update::ModelUpdateManager;
use crate::managers::storage::StorageManager;

use std::env; // 환경 변수 및 인자 수집용
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use sysinfo::System;
use tauri::{AppHandle, Emitter, Manager, Url, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_deep_link::DeepLinkExt; // 딥 링크 확장 트레이트

// --- 3. 전역 상태 타입 정의 ---

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActiveSessionInfo {
    pub session_id: String,
    pub task_id: Option<String>,
    pub start_time_s: u64, // Unix timestamp (seconds)
}

//  MainView.tsx가 invoke할 Task 데이터 모델 (handlers.ts 미러링) --- 중간 점검 production
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
    id: String,
    user_id: String,
    task_name: String,
    description: String,
    due_date: String,
    status: String,
    target_executable: String,
    target_arguments: Vec<String>, // handlers.ts는 [] (빈 배열)이었음
    created_at: String,
    updated_at: String,
}

// LSN 이벤트 캐싱을 위한 통합 데이터 모델 (stroage manger.rs)
pub struct LoggableEventData<'a> {
    pub app_name: &'a str,
    pub window_title: &'a str,
    pub input_stats: &'a commands::input::InputStats,
    // [추후] pub current_url: Option<&'a str>,
}

// 애플리케이션 전역에서 공유할 시스템 정보 상태 정의
pub struct SysinfoState(pub Mutex<System>);

// 사용자 입력 통계 추적을 위한 공유 상태
pub type InputStatsArcMutex = Arc<Mutex<commands::input::InputStats>>;

// StateEngine을 전역 상태로 관리하기 위한 타입 정의
pub type StateEngineArcMutex = Arc<Mutex<core::state::StateEngine>>;

// 전역 LSN(StorageManager) 상태 타입
pub type StorageManagerArcMutex = Arc<Mutex<StorageManager>>;

// 전역 세션 상태
pub type SessionStateArcMutex = Arc<Mutex<Option<ActiveSessionInfo>>>;

// --- 공통 딥 링크 처리 함수 (핵심 로직 통합) ---
// Single Instance와 on_open_url 양쪽에서 호출합니다.
fn handle_deep_link(app: &AppHandle, url: &Url) {
    println!("Processing Deep Link: {}://{}{}...", url.scheme(), url.host_str().unwrap_or("?"), url.path());

    // 1. URL 구조 검증 (Host='auth', Path='/callback')
    let is_scheme_valid = url.scheme() == "force-focus";
    let is_host_valid = url.host_str() == Some("auth");
    let is_path_valid = url.path() == "/callback";

    if is_scheme_valid && is_host_valid && is_path_valid {
        let query_pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();

        if let (Some(access), Some(refresh), Some(email), Some(user_id)) = (
            query_pairs.get("access_token"),
            query_pairs.get("refresh_token"),
            query_pairs.get("email"),
            query_pairs.get("user_id"),
        ) {
            println!("Login detected for user: [REDACTED]");

            // 2. LSN 저장 (AppHandle을 통해 State 접근)
            if let Some(storage_state) = app.try_state::<StorageManagerArcMutex>() {
                // Mutex Lock
                match storage_state.lock() {
                    Ok(storage) => {
                        if let Err(e) = storage.save_auth_token(access, refresh, email, user_id) {
                            eprintln!("CRITICAL: Failed to save auth token to LSN: {}", e);
                        } else {
                            println!("Auth token saved to LSN successfully.");

                            // 3. 프론트엔드 알림 (화면 전환)
                            if let Err(e) = app.emit("login-success", email) {
                                eprintln!("Failed to emit login-success event: {}", e);
                            }

                            // 4. 메인 창 띄우기 (포커스)
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                if window.is_minimized().unwrap_or(false) {
                                    let _ = window.unminimize();
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("CRITICAL: Failed to lock storage manager mutex: {}", e),
                }
            } else {
                eprintln!("CRITICAL: StorageManager state not found in AppHandle.");
            }
        } else {
            eprintln!("Deep Link Error: Missing required query parameters (access/refresh/email)");
        }
    } else {
        println!(
            "Deep Link Skipped. Mismatch structure. Host={:?}, Path={}",
            url.host_str(),
            url.path()
        );
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let input_stats_manager_state: InputStatsArcMutex = Arc::new(Mutex::new(commands::input::InputStats::default()));
    let state_engine_manager_state: StateEngineArcMutex = Arc::new(Mutex::new(core::state::StateEngine::new()));
    let backend_communicator_state = Arc::new(utils::backend_comm::BackendCommunicator::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent, 
            Some(vec!["--silent"])
        ))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            println!("Single Instance: New instance detected.");
            if let Some(url_str) = argv.iter().find(|arg| arg.starts_with("force-focus://")) {
                if let Ok(url) = Url::parse(url_str) {
                    handle_deep_link(app, &url);
                } else {
                    eprintln!("Single Instance: Failed to parse URL: {}", url_str);
                }
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                if window.is_minimized().unwrap_or(false) {
                    let _ = window.unminimize();
                }
            }
        }))
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    println!("Window Close Requested: Hiding window to system tray.");
                    api.prevent_close(); 
                    let _ = window.hide();
                }
            }
        })
        .manage(commands::system::SysinfoState(Mutex::new(System::new_all())))
        .manage(input_stats_manager_state)
        .manage(state_engine_manager_state)
        .manage(backend_communicator_state)
        .setup(move |app| {
            setup_ml_engine(app)?;
            setup_storage_and_session(app)?;
            handle_app_startup(app)?;

            let app_handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    handle_deep_link(&app_handle, &url);
                }
            });

            start_background_services(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::commands::vision::get_current_active_window_info,
            crate::commands::system::get_all_processes_summary,
            crate::commands::input::get_input_frequency_stats,
            crate::commands::vision::get_visible_windows,
            crate::commands::ml::check_model_update,
            crate::utils::backend_comm::submit_feedback,
            crate::utils::backend_comm::start_session,
            crate::utils::backend_comm::end_session,
            crate::utils::backend_comm::get_tasks,
            crate::utils::backend_comm::get_current_session_info,
            crate::utils::backend_comm::login,
            crate::utils::backend_comm::logout,
            crate::utils::backend_comm::check_auth_status,
            crate::commands::window::hide_overlay,
            crate::commands::window::show_overlay,                   
            crate::commands::window::set_overlay_ignore_cursor_events, 
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_ml_engine(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();
    let app_data_dir = app_handle.path().app_data_dir().ok();
    
    let mut model_path_buf = PathBuf::new();
    if let Some(ref dir) = app_data_dir {
        model_path_buf.push(dir);
        model_path_buf.push("models");
        model_path_buf.push("personal_model.onnx");
    }

    let final_model_path = if model_path_buf.exists() {
        println!("Using updated model from AppData: {:?}", model_path_buf);
        model_path_buf.to_string_lossy().to_string()
    } else {
        println!("Using default embedded model (resources).");
        "resources/models/personal_model.onnx".to_string()
    };

    let scaler_path = "resources/models/scaler_params.json".to_string();

    match InferenceEngine::new(&final_model_path, &scaler_path) { 
        Ok(engine) => {
            println!("✅ ML Inference Engine Loaded.");
            app.manage(Mutex::new(engine));
        }
        Err(e) => {
            eprintln!("⚠️ Failed to load ML Engine: {}", e);
        }
    }

    let update_manager = ModelUpdateManager::new(app_handle.clone());
    app.manage(update_manager);
    ai::model_update::start_update_loop(app_handle.clone());

    Ok(())
}

fn setup_storage_and_session(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();
    let storage_manager = StorageManager::new_from_path(app_handle)
        .expect("Failed to initialize StorageManager (LSN)");

    let initial_session_state: Option<ActiveSessionInfo> = storage_manager.load_active_session().unwrap_or_else(|e| {
        eprintln!("Failed to load active session from LSN: {}. Starting clean.", e);
        None
    });

    app.manage(Arc::new(Mutex::new(initial_session_state)) as SessionStateArcMutex);
    app.manage(Arc::new(Mutex::new(storage_manager)) as StorageManagerArcMutex);

    Ok(())
}

fn handle_app_startup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    println!("Startup Args: {:?}", args);
    let is_silent = args.iter().any(|arg| arg == "--silent");

    if let Some(window) = app.get_webview_window("main") {
        if is_silent {
            println!("App started in silent mode (Tray only).");
        } else {
            println!("App started normally. Showing main window.");
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
    Ok(())
}

fn start_background_services(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();
    
    let session_manager_state = app.state::<SessionStateArcMutex>().inner().clone();
    let storage_manager_state = app.state::<StorageManagerArcMutex>().inner().clone();
    let input_stats_manager_state = app.state::<InputStatsArcMutex>().inner().clone();

    core::input::start_input_listener(input_stats_manager_state.clone());

    use crate::core::app::AppCore;
    app.manage(std::sync::Mutex::new(AppCore::new(&app_handle)));

    core::app::start_core_loop(
        app_handle.clone(),
        session_manager_state.clone(),
        storage_manager_state.clone(),
        input_stats_manager_state.clone(),
    );

    managers::tray::setup_tray_menu(&app_handle)?;

    managers::widget::setup_widget_listeners(
        app_handle.clone(),
        session_manager_state.clone(),
    );

    managers::sync::start_sync_loop(app_handle.clone());
    managers::schedule::start_monitor_loop(app_handle.clone());

    Ok(())
}

