// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod commands;
mod logging;

use tauri::{Manager, Builder, State};
use std::sync::{Mutex, Arc};
use sysinfo::System;

use rdev::{listen, Event, EventType};
use std::thread; // 백그라운드 스레드를 위해 필요
use std::time::{SystemTime, UNIX_EPOCH, Duration};


// 애플리케이션 전역에서 공유할 시스템 정보 상태 정의
pub struct SysinfoState(pub Mutex<System>);

// 사용자 입력 통계 추적을 위한 공유 상태
pub type InputStatsArcMutex = Arc<Mutex<commands::InputStats>>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

    // InputStats 초기화 데이터를 먼저 생성합니다.
    let initial_input_stats = commands::InputStats {
        total_input_events: 0,
        last_input_timestamp_ms: SystemTime::now().duration_since(UNIX_EPOCH)
                                        .unwrap_or_else(|_| Duration::from_secs(0))
                                        .as_millis() as u64,
        start_monitoring_timestamp_ms: SystemTime::now().duration_since(UNIX_EPOCH)
                                        .unwrap_or_else(|_| Duration::from_secs(0))
                                        .as_millis() as u64,
    };

    // InputStatsArcMutex 타입을 직접 manage 하도록 합니다.
    let input_stats_manager_state: InputStatsArcMutex = Arc::new(Mutex::new(initial_input_stats));


    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())

        .manage(commands::SysinfoState( // commands::SysinfoState로 경로 명시
            Mutex::new(System::new_all()),
        ))

        
        // Arc<Mutex<commands::InputStats>> 타입을 관리
        .manage(input_stats_manager_state.clone()) // 초기화된 Arc를 manage에 전달


        .setup(|app| {
            let app_handle = app.handle();
            let input_stats_arc_mutex_for_thread = Arc::clone(app_handle.state::<InputStatsArcMutex>().inner());

            // 백그라운드 스레드에서 rdev 이벤트를 리스닝하고,
            // input_stats_arc_mutex_for_thread를 업데이트합니다.
            start_input_listener(input_stats_arc_mutex_for_thread);

            
            // 데이터 수집 및 로깅 기능 시작
            let input_stats_arc_mutex_for_logging = Arc::clone(app_handle.state::<InputStatsArcMutex>().inner());
            logging::start_data_collection_and_logging(input_stats_arc_mutex_for_logging, 10); // 10초마다 로깅

    
            Ok(())
        })

        .invoke_handler(tauri::generate_handler![
            greet,
            commands::get_current_active_window_info,
            commands::get_all_processes_summary,
            commands::get_input_frequency_stats,

            ])

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}


// rdev 이벤트 리스너를 시작하는 함수.
fn start_input_listener(input_stats_arc_mutex: InputStatsArcMutex) {
    thread::spawn(move || {
        // rdev::listen 클로저 내부에서 input_stats_arc_mutex를 사용하도록 move
        if let Err(error) = listen(move |event| {
            match event.event_type {
                // 키보드 누름
                EventType::KeyPress(_) => {
                    let mut stats_guard = input_stats_arc_mutex.lock().unwrap();
                    stats_guard.total_input_events += 1;
                    stats_guard.last_input_timestamp_ms = SystemTime::now().duration_since(UNIX_EPOCH)
                                                    .unwrap_or_else(|_| Duration::from_secs(0))
                                                    .as_millis() as u64;
                    // (디버깅용) 키보드 이벤트 발생 시 콘솔 출력
                    // eprintln!("KeyPress/Release detected: {:?}", event.event_type);
                },
                // 마우스 버튼 누름 (마우스 휠 추가 예정)
                EventType::ButtonPress(_) => {
                    let mut stats_guard = input_stats_arc_mutex.lock().unwrap();
                    stats_guard.total_input_events += 1;
                    stats_guard.last_input_timestamp_ms = SystemTime::now().duration_since(UNIX_EPOCH)
                                                    .unwrap_or_else(|_| Duration::from_secs(0))
                                                    .as_millis() as u64;
                    // (디버깅용) 마우스 버튼 이벤트 발생 시 콘솔 출력
                    // eprintln!("ButtonPress/Release detected: {:?}", event.event_type);
                },
                // 마우스 이동, 휠 등 다른 이벤트는 무시
                _ => (),
            }
        }) {
            eprintln!("Error listening for input events: {:?}", error);
        }
    });
}