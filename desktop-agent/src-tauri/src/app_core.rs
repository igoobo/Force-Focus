// 파일 위치: src-tauri/src/app_core.rs

use tauri::{AppHandle, Manager, State, Runtime, Emitter, WebviewWindowBuilder, WebviewUrl};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH}; 
use crate::{
    commands, // commands 모듈의 _get_active_window_info_internal 사용
    state_engine, // state_engine 모듈 사용
    InputStatsArcMutex, // lib.rs에서 정의한 타입
    StateEngineArcMutex, // lib.rs에서 정의할 타입
    SessionStateArcMutex, // 전역 세션 상태 import
    StorageManagerArcMutex, // LSN import (이벤트 캐싱을 위해)
};

/// '메인 루프'를 별도 스레드에서 시작
/// 이 루프는 5초마다 StateEngine을 실행
pub fn start_core_loop<R: Runtime>(
    app_handle: AppHandle<R>,
    session_state_mutex: SessionStateArcMutex, // 세션 상태 인자
    storage_manager_mutex: StorageManagerArcMutex, // LSN 인자
) {
    let mut state_engine_counter = 0;


    thread::spawn(move || {
        loop {
            // 5초마다 실행
            thread::sleep(Duration::from_secs(1));
            
            state_engine_counter += 1;
            let now_s = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());

            // 세션 활성화 상태 검사
            let session_guard = session_state_mutex.lock().unwrap();

            
            if let Some(active_session) = &*session_guard {
                // 세션이 활성 상태일 때만 아래 로직을 실행

                // [추가] Task 4.12 (P1): Rust에서 타이머 계산
                let elapsed_seconds = now_s.saturating_sub(active_session.start_time_s);
                
                // [수정] Task 4.12 (P1): 'widget-tick' 이벤트를 '모든' 창에 방송(emit)
                // [!] (v2 API) app_handle.emit()은 'broadcast'입니다.
                app_handle.emit("widget-tick", elapsed_seconds).ok();


                if state_engine_counter >= 5 {
                    state_engine_counter = 0; // 카운터 리셋
                    
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
                     // 보이는 창 목록 데이터 수집   반환값: Vec<WindowInfo>
                    let visible_windows = commands::_get_all_visible_windows_internal();


                    // --- 2. 센서 데이터 수집 (Input Monitor) ---
                    let input_stats_state: State<'_, InputStatsArcMutex> = app_handle.state();
                    let mut input_stats = input_stats_state.lock().unwrap(); // Mutex 잠금
                    

                    // [추가] Task 2.2: 수집된 시각 데이터를 InputStats 구조체에 채워 넣음
                    // (WindowInfo 구조체에서 title만 추출하여 String 벡터로 변환)
                    // [!] ML 모델을 위해 '전경 여부'도 포함할 수 있지만, 현재는 title만 저장
                    input_stats.visible_windows = visible_windows;


                    // InputStats를 JSON 문자열로 직렬화 (commands.rs 헬퍼 호출)
                    let activity_vector_json = input_stats.to_activity_vector_json();

                    
                    // 단순화된 LSN API를 호출합니다.
                    let storage_manager = storage_manager_mutex.lock().unwrap();
                    storage_manager.cache_event(
                        &active_session.session_id, 
                        &window_info.app_name, 
                        &window_info.title,
                        &activity_vector_json // JSON 문자열 전달
                    ).unwrap_or_else(|e| eprintln!("Failed to cache event: {}", e));
                    drop(storage_manager); 


                    // --- 3. StateEngine에 데이터 주입 ---
                    let engine_state: State<'_, StateEngineArcMutex> = app_handle.state();
                    let mut engine = engine_state.lock().unwrap(); // Mutex 잠금 (변경을 위해 mut)

                    let trigger = engine.process_activity(&window_info, &input_stats);

                    drop(engine); // StateEngine 락 즉시 해제
                    drop(input_stats); // InputStats 락 즉시 해제
                    
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
            } else {
                // --- [B] 세션이 비활성 상태일 때 ---
                state_engine_counter = 0; // 카운터 리셋
                
                // [추가] Task 4.12: 'widget-tick'을 0으로 방송
                app_handle.emit("widget-tick", 0).ok();
            }
        drop(session_guard); // 세션 락 해제
        }
    });
}