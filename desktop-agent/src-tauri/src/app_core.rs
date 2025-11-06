// 파일 위치: src-tauri/src/app_core.rs

use tauri::{AppHandle, Manager, State, Runtime, Emitter, WebviewWindowBuilder, WebviewUrl};
use std::thread;
use std::time::Duration;
use crate::{
    commands, // commands 모듈의 _get_active_window_info_internal 사용
    state_engine, // state_engine 모듈 사용
    InputStatsArcMutex, // lib.rs에서 정의한 타입
    StateEngineArcMutex, // lib.rs에서 정의할 타입
};

/// '메인 루프'를 별도 스레드에서 시작
/// 이 루프는 5초마다 StateEngine을 실행
pub fn start_core_loop(app_handle: AppHandle) {
    thread::spawn(move || {
        loop {
            // 5초마다 실행
            thread::sleep(Duration::from_secs(5));

            // --- 1. 센서 데이터 수집 (Activity Monitor) ---
            let window_info_result = commands::_get_active_window_info_internal();

            // 활성 창 정보 수집에 실패하면 이번 루프는 무시
            let window_info = match window_info_result {
                Ok(info) => info,
                Err(e) => {
                    eprintln!("Core Loop Error (WindowInfo): {:?}", e);
                    continue; // 다음 루프 실행
                }
            };

            // --- 2. 센서 데이터 수집 (Input Monitor) ---
            let input_stats_state: State<'_, InputStatsArcMutex> = app_handle.state();
            let input_stats = input_stats_state.lock().unwrap(); // Mutex 잠금

            // --- 3. StateEngine에 데이터 주입 ---
            let engine_state: State<'_, StateEngineArcMutex> = app_handle.state();
            let mut engine = engine_state.lock().unwrap(); // Mutex 잠금 (변경을 위해 mut)

            let trigger = engine.process_activity(&window_info, &input_stats);

            // Mutex 잠금을 최대한 빨리 해제하기 위해 'input_stats'의 복사본을 만들거나,
            // 이 블록 안에서 모든 작업을 마칩니다.
            // 여기서는 'engine'의 잠금이 끝났으므로 다음 단계로 넘어갑니다.
            
            // (input_stats, engine의 MutexGuard는 여기서 자동으로 해제됨)

            // --- 4. 개입 컨트롤러 (Intervention Controller) ---
            // StateEngine의 결정에 따라 프론트엔드로 이벤트를 전송
            match trigger {
                state_engine::InterventionTrigger::TriggerNotification => {
                    println!("Core Loop: Triggering Notification"); // (디버깅용)
                    // 프론트엔드의 'intervention-trigger' 리스너를 호출
                    app_handle.emit("intervention-trigger", "notification")
                        .unwrap_or_else(|e| eprintln!("Failed to emit event: {:?}", e));
                }
                state_engine::InterventionTrigger::TriggerOverlay => {
                    // '강한 개입'은 Rust가 직접 네이티브 창을 제어
                    println!("Core Loop: Triggering Overlay (Native)");
                    
                    // tauri.conf.json에 정의된 "overlay" 창 찾기
                    // 2. 'Get-or-Create' 로직
                    if let Some(overlay_window) = app_handle.get_webview_window("overlay") {
                        // --- [케이스 1] 창이 존재함 (정상) ---
                        // (숨겨진 창을 다시 띄우고 포커스)
                        if let Err(e) = overlay_window.show() {
                            eprintln!("Failed to show overlay window: {:?}", e);
                        }
                        if let Err(e) = overlay_window.set_focus() {
                             eprintln!("Failed to focus overlay window: {:?}", e);
                        }
                    } else {
                        // --- [케이스 2] 창이 없음 (Alt+F4로 파괴됨) ---
                        // (tauri.conf.json과 동일한 설정으로 창을 재생성)
                        println!("Core Loop: Overlay window not found. Re-creating...");
                        if let Err(e) = WebviewWindowBuilder::new(
                            &app_handle,
                            "overlay", // 1. 고유 레이블
                            WebviewUrl::App("overlay.html".into()) // 2. HTML 경로
                        )
                        .fullscreen(true)
                        .decorations(false)
                        .transparent(true)
                        .always_on_top(true)
                        .skip_taskbar(true)
                        .resizable(false)
                        .visible(true) // 3. 생성과 동시에 'show'
                        .build()
                        {
                            eprintln!("Failed to re-create overlay window: {:?}", e);
                        }
                    }
                }
                state_engine::InterventionTrigger::DoNothing => {
                    // 아무것도 하지 않음
                }
            }
        }
    });
}