// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod commands;
mod logging;
pub mod input_monitor;
pub mod state_engine;
pub mod app_core;
pub mod backend_communicator;
pub mod window_commands;
pub mod storage_manager;
pub mod widget_manager; 
pub mod tray_manager; 

use tauri::{Manager, Builder, State};
use std::sync::{Mutex, Arc};
use sysinfo::System;

use std::time::{SystemTime, UNIX_EPOCH, Duration};

use crate::storage_manager::StorageManager; 

// ---  전역 공유 데이터 모델 
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

// LSN 이벤트 캐싱을 위한 통합 데이터 모델
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
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())

        .manage(commands::SysinfoState( // commands::SysinfoState로 경로 명시
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
            let storage_manager =
                StorageManager::new_from_path(app_handle.clone())
                    .expect("Failed to initialize StorageManager (LSN)");
            
            // '활성 세션'을 로드하여 전역 상태 초기화
            let initial_session_state: Option<ActiveSessionInfo> =
                storage_manager.load_active_session().unwrap_or_else(|e| {
                    eprintln!("Failed to load active session from LSN: {}. Starting clean.", e);
                    None
                });

            
            let session_manager_state: SessionStateArcMutex = Arc::new(Mutex::new(initial_session_state));
            let storage_manager_state: StorageManagerArcMutex = Arc::new(Mutex::new(storage_manager));

            // LSN(StorageManager)을 전역 상태로 등록
            app.manage(storage_manager_state.clone());

            app.manage(session_manager_state.clone());


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
                storage_manager_state.clone(),  // LSN 전달
            );
            
            tray_manager::setup_tray_menu(app.handle())?;

             // --- [추가] Task 4.10: '위젯 관리' 모듈 초기화 ---
            widget_manager::setup_widget_listeners(
                app_handle.clone(), 
                session_manager_state.clone()
            );

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

            window_commands::hide_overlay
            ])

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
