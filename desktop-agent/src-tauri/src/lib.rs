// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// --- 1. 모듈 선언 ---
pub mod app_core;
pub mod backend_communicator;
mod commands;
pub mod input_monitor;
mod logging;
pub mod state_engine;
#[allow(dead_code)]
pub mod storage_manager;
pub mod tray_manager;
pub mod widget_manager;
pub mod window_commands;
pub mod sync_manager;

// --- 2. 전역 use ---

use crate::storage_manager::StorageManager;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::System;
use tauri::{Builder, Emitter, Manager, State, Url, AppHandle};
use tauri_plugin_deep_link::DeepLinkExt; //  딥 링크 확장 트레이트

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
    pub input_stats: &'a commands::InputStats,
    // [추후] pub current_url: Option<&'a str>,
}

// 애플리케이션 전역에서 공유할 시스템 정보 상태 정의
pub struct SysinfoState(pub Mutex<System>);

// 사용자 입력 통계 추적을 위한 공유 상태
pub type InputStatsArcMutex = Arc<Mutex<commands::InputStats>>;

// StateEngine을 전역 상태로 관리하기 위한 타입 정의
pub type StateEngineArcMutex = Arc<Mutex<state_engine::StateEngine>>;

// 전역 LSN(StorageManager) 상태 타입
pub type StorageManagerArcMutex = Arc<Mutex<StorageManager>>;

// 전역 세션 상태
pub type SessionStateArcMutex = Arc<Mutex<Option<ActiveSessionInfo>>>;


// --- 공통 딥 링크 처리 함수 (핵심 로직 통합) ---
// Single Instance와 on_open_url 양쪽에서 호출합니다.
fn handle_deep_link(app: &AppHandle, url: &Url) {
    println!("Processing Deep Link: {}", url);

    // 1. URL 구조 검증 (Host='auth', Path='/callback')
    let is_scheme_valid = url.scheme() == "force-focus";
    let is_host_valid = url.host_str() == Some("auth");
    let is_path_valid = url.path() == "/callback";

    if is_scheme_valid && is_host_valid && is_path_valid {
        let query_pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        
        if let (Some(access), Some(refresh), Some(email)) = (
            query_pairs.get("access_token"),
            query_pairs.get("refresh_token"),
            query_pairs.get("email")
        ) {
            println!("Login detected for user: {}", email);
            
            // 2. LSN 저장 (AppHandle을 통해 State 접근)
            if let Some(storage_state) = app.try_state::<StorageManagerArcMutex>() {
                // Mutex Lock
                match storage_state.lock() {
                    Ok(storage) => {
                        if let Err(e) = storage.save_auth_token(access, refresh, email) {
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
                    },
                    Err(e) => eprintln!("CRITICAL: Failed to lock storage manager mutex: {}", e),
                }
            } else {
                eprintln!("CRITICAL: StorageManager state not found in AppHandle.");
            }
        } else {
            eprintln!("Deep Link Error: Missing required query parameters (access/refresh/email)");
        }
    } else {
        println!("Deep Link Skipped. Mismatch structure. Host={:?}, Path={}", url.host_str(), url.path());
    }
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // InputStats 초기화 데이터를 먼저 생성
    let initial_input_stats = commands::InputStats::default();

    // InputStatsArcMutex 타입을 직접 manage
    let input_stats_manager_state: InputStatsArcMutex = Arc::new(Mutex::new(initial_input_stats));

    // BackendCommunicator 인스턴스를 생성
    let backend_communicator_state = backend_communicator::BackendCommunicator::new();

    // StateEngine 인스턴스를 생성
    let state_engine_manager_state: StateEngineArcMutex =
        Arc::new(Mutex::new(state_engine::StateEngine::new()));

    // Offline-First를 위한 상태 생성
    let backend_communicator_state = Arc::new(backend_communicator::BackendCommunicator::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init()) // 딥 링크 플러그인 초기화
        .plugin(tauri_plugin_opener::init())

        // 단일 인스턴스 플러그인 (Windows 딥 링크 해결사)
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            println!("Single Instance: New instance detected.");
            // 1. URL 찾기
            let deep_link_url = argv.iter().find(|arg| arg.starts_with("force-focus://"));

            if let Some(url_str) = deep_link_url {
                // 2. 파싱 후 공통 함수 호출
                if let Ok(url) = Url::parse(url_str) {
                    handle_deep_link(app, &url);
                } else {
                    eprintln!("Single Instance: Failed to parse URL: {}", url_str);
                }
            }
            
            // (딥 링크가 아니더라도) 창을 앞으로 띄움
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                if window.is_minimized().unwrap_or(false) {
                    let _ = window.unminimize();
                }
            }
        }))

        .manage(commands::SysinfoState(
            // commands::SysinfoState로 경로 명시
            Mutex::new(System::new_all()),
        ))
        // Arc<Mutex<commands::InputStats>> 타입을 관리
        .manage(input_stats_manager_state.clone()) // 초기화된 Arc를 manage에 전달
        // StateEngine을 전역 상태로 등록
        .manage(state_engine_manager_state.clone())
        // BackendCommunicator를 전역 상태로 등록
        .manage(backend_communicator_state)
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // --- LSN 초기화 및 등록 ---
            let storage_manager = StorageManager::new_from_path(app_handle.clone())
                .expect("Failed to initialize StorageManager (LSN)");

            // '활성 세션'을 로드하여 전역 상태 초기화
            let initial_session_state: Option<ActiveSessionInfo> =
                storage_manager.load_active_session().unwrap_or_else(|e| {
                    eprintln!(
                        "Failed to load active session from LSN: {}. Starting clean.",
                        e
                    );
                    None
                });

            let session_manager_state: SessionStateArcMutex =
                Arc::new(Mutex::new(initial_session_state));
            let storage_manager_state: StorageManagerArcMutex =
                Arc::new(Mutex::new(storage_manager));

            // LSN(StorageManager)을 전역 상태로 등록
            app.manage(storage_manager_state.clone());

            app.manage(session_manager_state.clone());

            // [macOS용] Deep Link 리스너 (Windows는 single-instance가 처리하지만, macOS는 이게 필요함)
            // 'storage_manager_state'를 클로저 내부로 안전하게 이동시키기 위해 clone
            let storage_manager_for_deep_link = storage_manager_state.clone();
            let app_handle_for_deep_link = app_handle.clone();

            // ---  Deep Link 리스너 (공통 함수 사용) ---
            // setup 훅 내부에서 실행되는 리스너
            let value = app_handle.clone();
            app.deep_link().on_open_url(move |event| {
                let urls = event.urls();
                for url in urls {
                    // url은 이미 tauri::Url 객체
                    handle_deep_link(&value, &url);
                }
            });

            // rdev 이벤트 리스너를 별도의 스레드에서 시작하는 함수
            input_monitor::start_input_listener(input_stats_manager_state.clone());

            // // 데이터 수집 및 로깅 기능 시작
            // let input_stats_arc_mutex_for_logging = Arc::clone(app_handle.state::<InputStatsArcMutex>().inner());
            // logging::start_data_collection_and_logging(input_stats_arc_mutex_for_logging, 10); // 10초마다 로깅

            // app_core의 '메인 루프'를 시작
            // app_handle을 복제하여 넘겨주어 스레드가 AppHandle을 소유
            app_core::start_core_loop(
                app_handle.clone(),
                session_manager_state.clone(), // 세션 상태 전달
                storage_manager_state.clone(), // LSN 전달
            );

            tray_manager::setup_tray_menu(app.handle())?;

            // --- [추가] Task 4.10: '위젯 관리' 모듈 초기화 ---
            widget_manager::setup_widget_listeners(
                app_handle.clone(),
                session_manager_state.clone(),
            );

            // --- 백그라운드 데이터 동기화 시작 ---
            // 1분마다 LSN 데이터를 서버로 전송하는 루프를 시작
            // (내부적으로 토큰이 없으면 건너뛰므로 안전)
            sync_manager::start_sync_loop(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::get_current_active_window_info,
            commands::get_all_processes_summary,
            commands::get_input_frequency_stats,
            commands::get_visible_windows, // 시각 센서 커맨드 등록
            // backend_communicator 모듈의 커맨드를 핸들러에 등록
            backend_communicator::submit_feedback,
            backend_communicator::start_session,
            backend_communicator::end_session,
            backend_communicator::get_tasks,
            backend_communicator::get_current_session_info,
            backend_communicator::login,  //  로그인 커맨드
            backend_communicator::logout, //  로그아웃 커맨드
            backend_communicator::check_auth_status, // 자동 로그인 커맨드 등록
            window_commands::hide_overlay
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
