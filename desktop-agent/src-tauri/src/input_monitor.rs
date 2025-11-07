// 파일 위치: src-tauri/src/input_monitor.rs

use crate::InputStatsArcMutex; // lib.rs (crate root)에서 정의한 타입
use rdev::{listen, Event, EventType};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// rdev 이벤트 리스너를 별도의 스레드에서 시작하는 함수.
pub fn start_input_listener(input_stats_arc_mutex: InputStatsArcMutex) {
    thread::spawn(move || {
        if let Err(error) = listen(move |event| {
            // 이벤트 발생 시간
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_millis() as u64;

            match event.event_type {
                // 유의미한 입력 (키/클릭/휠)
                EventType::KeyPress(_) |
                EventType::ButtonPress(_) |
                EventType::Wheel { .. } => {
                    update_meaningful_stats(&input_stats_arc_mutex, now_ms);
                }
                
                // 마우스 이동 (분리)
                EventType::MouseMove { .. } => {
                    update_mouse_move_stats(&input_stats_arc_mutex, now_ms);
                }
                
                _ => (), // 다른 이벤트 무시
            }
        }) {
            eprintln!("Error listening for input events: {:?}", error);
        }
    });
}

/// '유의미한 입력' 통계를 갱신하는 헬퍼 함수
fn update_meaningful_stats(input_stats_arc_mutex: &InputStatsArcMutex, now_ms: u64) {
    if let Ok(mut stats_guard) = input_stats_arc_mutex.lock() {
        stats_guard.meaningful_input_events += 1;
        stats_guard.last_meaningful_input_timestamp_ms = now_ms;
    }
}

/// '마우스 이동' 통계를 갱신하는 헬퍼 함수
fn update_mouse_move_stats(input_stats_arc_mutex: &InputStatsArcMutex, now_ms: u64) {
    if let Ok(mut stats_guard) = input_stats_arc_mutex.lock() {
        stats_guard.last_mouse_move_timestamp_ms = now_ms;
    }
}