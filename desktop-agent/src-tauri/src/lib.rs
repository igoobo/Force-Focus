// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod commands;
mod logging;
pub mod input_monitor;
pub mod state_engine;
pub mod app_core;
pub mod backend_communicator;
pub mod window_commands;
pub mod storage_manager;

use tauri::{Manager, Builder, State};
use std::sync::{Mutex, Arc};
use sysinfo::System;

use std::time::{SystemTime, UNIX_EPOCH, Duration};

// ---  전역 공유 데이터 모델 
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActiveSessionInfo {
    pub session_id: String,
    pub task_id: Option<String>,
    pub start_time_s: u64, // Unix timestamp (seconds)
}


// 애플리케이션 전역에서 공유할 시스템 정보 상태 정의
pub struct SysinfoState(pub Mutex<System>);

// 사용자 입력 통계 추적을 위한 공유 상태
pub type InputStatsArcMutex = Arc<Mutex<commands::InputStats>>;

// 2. StateEngine을 전역 상태로 관리하기 위한 타입 정의
pub type StateEngineArcMutex = Arc<Mutex<state_engine::StateEngine>>;

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

        .setup(|app| {
            let app_handle = app.handle();
            let input_stats_arc_mutex_for_thread = Arc::clone(app_handle.state::<InputStatsArcMutex>().inner());

            // rdev 이벤트 리스너를 별도의 스레드에서 시작하는 함수
            input_monitor::start_input_listener(input_stats_arc_mutex_for_thread);

            
            // // 데이터 수집 및 로깅 기능 시작
            // let input_stats_arc_mutex_for_logging = Arc::clone(app_handle.state::<InputStatsArcMutex>().inner());
            // logging::start_data_collection_and_logging(input_stats_arc_mutex_for_logging, 10); // 10초마다 로깅

            // app_core의 '메인 루프'를 시작
            // app_handle을 복제하여 넘겨주어 스레드가 AppHandle을 소유
            app_core::start_core_loop(app_handle.clone());

            Ok(())
        })

        .invoke_handler(tauri::generate_handler![
            greet,
            commands::get_current_active_window_info,
            commands::get_all_processes_summary,
            commands::get_input_frequency_stats,

            // backend_communicator 모듈의 커맨드를 핸들러에 등록
            backend_communicator::submit_feedback,

            window_commands::hide_overlay
            ])

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
