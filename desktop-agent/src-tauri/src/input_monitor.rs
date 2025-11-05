// 파일 위치: src-tauri/src/input_monitor.rs

use crate::InputStatsArcMutex; // lib.rs (crate root)에서 정의한 타입
use rdev::{listen, Event, EventType};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// rdev 이벤트 리스너를 별도의 스레드에서 시작하는 함수.
pub fn start_input_listener(input_stats_arc_mutex: InputStatsArcMutex) {
    thread::spawn(move || {
        // rdev::listen 클로저 내부에서 input_stats_arc_mutex를 사용하도록 move
        if let Err(error) = listen(move |event| {
            match event.event_type {
                // 키보드 누름
                EventType::KeyPress(_) => {
                    update_stats(&input_stats_arc_mutex);
                }
                // 마우스 버튼 누름 (마우스 휠은 rdev에서 ButtonPress로 오지 않을 수 있음, 확인 필요)
                EventType::ButtonPress(_) => {
                    update_stats(&input_stats_arc_mutex);
                }
                // 마우스 휠
                EventType::Wheel { .. } => {
                    update_stats(&input_stats_arc_mutex);
                }

                 // 마우스 이동을 감지
                EventType::MouseMove { .. } => {
                    update_stats(&input_stats_arc_mutex);
                }
                
                // 키 떼기 등 다른 이벤트는 무시
                _ => (),
            }
        }) {
            eprintln!("Error listening for input events: {:?}", error);
        }
    });
}

/// Mutex를 잠그고 입력 통계를 갱신하는 헬퍼 함수
fn update_stats(input_stats_arc_mutex: &InputStatsArcMutex) {
    if let Ok(mut stats_guard) = input_stats_arc_mutex.lock() {
        stats_guard.total_input_events += 1;
        stats_guard.last_input_timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_millis() as u64;
    }
}