// 파일 위치: src-tauri/src/app_core.rs

use crate::{
    commands::{self, ActiveWindowInfo, WindowInfo}, // commands 모듈 활용
    state_engine::{self, StateEngine, InterventionTrigger},
    InputStatsArcMutex,     // lib.rs에서 정의한 타입
    SessionStateArcMutex,   // 전역 세션 상태 import
    StateEngineArcMutex,    // lib.rs에서 정의할 타입
    StorageManagerArcMutex, // LSN import (이벤트 캐싱을 위해)
    inference::InferenceEngine // 추론 엔진
};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Runtime, State, WebviewUrl, WebviewWindowBuilder};

// ================================================================
// [Core Struct] 중앙 관제소 AppCore
// ================================================================
pub struct AppCore {
    // 1. 뇌 (ML)
    pub inference_engine: Option<InferenceEngine>,
    
    // 2. 심장 (FSM)
    pub state_engine: StateEngine,
    
    // 3. 눈 (데이터 수집 상태 기억)    
    pub last_event_count: u64,
    
    // 4. ML의 최근 판단 결과를 기억 (5초간 유지용)
    pub last_inference_result: crate::inference::InferenceResult,
}

impl AppCore {
    pub fn new() -> Self {
        // Step 5 전이라 모델 파일이 없을 수 있음 (Graceful Handling)
        let model_path = "resources/models/personal_model.onnx";
        let scaler_path = "resources/models/scaler_params.json";

        // 모델 로딩 시도 (실패 시 더미/에러 처리하되 앱은 안 죽게)
        let inference_engine = match InferenceEngine::new(model_path, scaler_path) {
            Ok(engine) => Some(engine),
            Err(e) => {
                // 경고만 출력하고 앱은 살려둠
                eprintln!("⚠️ [AppCore] Running without ML Model: {}", e);
                eprintln!("⚠️ (This is normal if you haven't run Step 5 yet)");
                None 
            }
        };

        Self {
            inference_engine,
            state_engine: StateEngine::new(),
            last_event_count: 0,
            last_inference_result: crate::inference::InferenceResult::Inlier, // 초기값
        }
    }
}

// ================================================================
// [Main Loop] 1초 주기 FSM + 5초 주기 센싱
// ================================================================
pub fn start_core_loop<R: Runtime>(
    app_handle: AppHandle<R>,
    session_state_mutex: SessionStateArcMutex,
    storage_manager_mutex: StorageManagerArcMutex,
    input_stats_mutex: InputStatsArcMutex,
) {
    let app_handle_clone = app_handle.clone();

    thread::spawn(move || {
        let mut tick_counter = 0; // 5초 주기 체크용

        loop {
            // 1. 기본 주기: 1초
            thread::sleep(Duration::from_secs(1));
            
            // [Critical] AppCore 락 획득
            // (lib.rs에서 manage하지 않았다면 여기서 에러가 나므로, 순서가 중요함)
            let app_core_state = app_handle_clone.state::<Mutex<AppCore>>();
            let mut core = match app_core_state.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    eprintln!("Failed to lock AppCore: {}", e);
                    continue;
                }
            };

            
            let now_ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

            // 밀리초 단위 시간 (Silence 계산용)
            let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;

            // 2. 세션 활성 체크
            let session_guard = session_state_mutex.lock().unwrap();
            // 가드를 통해 내부 데이터를 '복제(Clone)'한 뒤, 가드는 즉시 놓아줍니다.
            // ActiveSessionInfo는 Clone 트레이트가 있어야 합니다. (보통 derive로 되어 있음)
            let active_session_opt = session_guard.clone(); 
            drop(session_guard); // 락 해제 (이제 안전함)

            if let Some(active_session) = active_session_opt { // 복제된 데이터를 소유권(Owned) 형태로 사용

                // 타이머 방송
                let elapsed = now_ts.saturating_sub(active_session.start_time_s);
                app_handle_clone.emit("widget-tick", elapsed).ok(); // 'widget-tick' 이벤트를 '모든' 창에 방송(emit)

                tick_counter += 1;

                // ------------------------------------------------
                // [Fast Path] 1초마다 실행 (가벼운 데이터)
                // ------------------------------------------------
                let mut input_stats = input_stats_mutex.lock().unwrap();
                let current_events = input_stats.meaningful_input_events;
                
                // Safety Net용 활동 감지
                let has_recent_input = (now_ms.saturating_sub(input_stats.last_meaningful_input_timestamp_ms) < 2000);
                let is_mouse_active = (now_ms.saturating_sub(input_stats.last_mouse_move_timestamp_ms) < 2000);

                // ------------------------------------------------
                // [Slow Path] 5초마다 실행 (무거운 센싱 & ML)
                // ------------------------------------------------
                if tick_counter >= 5 {
                    tick_counter = 0; // 카운터 리셋

                    // 1. 활성 창 정보 수집
                    if let Ok(window_info) = commands::_get_active_window_info_internal() {
                        
                        // 시각 센서 (Visible Windows) 수집
                        let mut visible_windows_raw = commands::_get_all_visible_windows_internal();

                        // 시맨틱 태깅 (Semantic Tagging)
                        // 원본 제목을 '토큰화 + 숫자 필터링'된 문자열로 세탁
                        for window in &mut visible_windows_raw {
                            let tokens = commands::get_semantic_tokens(&window.app_name, &window.title);
                            if !tokens.is_empty() {
                                window.title = tokens.join(" ");
                            } else {
                                window.title = String::new(); // 개인정보 보호
                            }
                        }

                        // 활성 창(Active Window) 태깅
                        // 활성 창 역시 동일한 로직으로 토큰을 추출합니다.
                        let active_tokens = commands::get_semantic_tokens(&window_info.app_name, &window_info.title);
                        let sanitized_active_title = active_tokens.join(" ");

                        // InputStats에 시각 데이터 업데이트
                        // [!] ML 모델을 위해 '전경 여부'도 포함할 수 있지만, 현재는 title만 저장
                        input_stats.visible_windows = visible_windows_raw;
                        // InputStats를 JSON 문자열로 직렬화 (commands.rs 헬퍼 호출)
                        let activity_vector_json = input_stats.to_activity_vector_json(); // LSN 저장용

                        // 2. ML Feature 생성 (Delta Event 등)
                        let delta_events = current_events.saturating_sub(core.last_event_count);
                        core.last_event_count = current_events; // 상태 업데이트
                    
                        let silence_sec = if input_stats.last_meaningful_input_timestamp_ms > 0 {
                            (now_ms.saturating_sub(input_stats.last_meaningful_input_timestamp_ms) as f64) / 1000.0
                        } else { 0.0 };

                        // 3. ML 벡터 구성
                        // [Context, LogInput, Silence, Burstiness, Mouse, Interaction]
                        let ml_vector = [
                            0.5, // Context (나중에 구현)
                            if delta_events > 0 { (delta_events as f64).ln() } else { 0.0 }, 
                            silence_sec,
                            0.0, 0.0, 0.0 
                        ];

                        
                        // 4. 데이터 저장 (학습용 데이터셋 구축)
                        // LSN에 이벤트를 저장해야 나중에 꺼내서 학습할 수 있습니다.
                        let storage = storage_manager_mutex.lock().unwrap();
                        let raw_json = serde_json::json!({
                            "delta_events": delta_events,
                            "silence_sec": silence_sec,
                            "window_title": window_info.title, // 원본 제목 (학습용)
                            "ml_vector": ml_vector
                        }).to_string();

                        storage
                            .cache_event(
                                &active_session.session_id,
                                &window_info.app_name,
                                &sanitized_active_title,
                                &activity_vector_json, // JSON 문자열 전달
                            )
                            .unwrap_or_else(|e| eprintln!("Failed to cache event: {}", e));
                        drop(storage);

                        // 5. ML 추론 (모델이 준비된 경우)
                        if let Some(engine) = &mut core.inference_engine {
                            match engine.infer(ml_vector, Some(window_info.app_name.clone())) {
                                Ok((score, judgment)) => {
                                    println!("🧠 ML: {:?} (Score: {:.3})", judgment, score);
                                    core.last_inference_result = judgment;
                                },
                                Err(e) => eprintln!("ML Inference Error: {}", e),
                            }
                        } else {
                            // 모델이 없으면 그냥 로그만 남김 (데이터 수집은 위에서 이미 끝남)
                            // println!("(ML skipped - model missing)");
                        }
                        
                    }
                }
                
                drop(input_stats); // InputStats 락 해제

                // ------------------------------------------------
                // FSM Update (매 1초마다 수행)
                // ------------------------------------------------
                // 5초간 업데이트된 'last_inference_result'를 사용하여 적분 제어

                // core(mutable)를 빌리기 전에, 필요한 데이터(immutable)를 미리 복사해둡니다.
                // InferenceResult는 Enum이므로 Clone 비용이 매우 쌉니다.
                let current_inference_result = core.last_inference_result.clone();

                let trigger = core.state_engine.process(
                    &current_inference_result, // 복사본 전달
                    now_ts,
                    is_mouse_active,
                    has_recent_input
                );

                // ------------------------------------------------
                // [Action] 개입 실행
                // ------------------------------------------------
                match trigger {
                    InterventionTrigger::TriggerNotification => {
                        println!("🔔 Notification");
                        app_handle_clone.emit("intervention-trigger", "notification").ok();
                    },
                    InterventionTrigger::TriggerOverlay => {
                        println!("🚫 Overlay");
                        trigger_overlay(&app_handle_clone);
                    },
                    InterventionTrigger::DoNothing => {}
                }

                
                    
            } else {
                // --- [B] 세션이 비활성 상태일 때 ---
                tick_counter = 0; // 카운터 리셋
                
                
                // (게이지가 0.0보다 클 때만 리셋 함수를 호출하여 로그 스팸 방지)
                if core.state_engine.get_gauge_ratio() > 0.0 {
                     core.state_engine.manual_reset(); 
                }
                
                // (혹시 모를 UI 동기화를 위해 0 전송은 유지)
                app_handle_clone.emit("widget-tick", 0).ok();
            }
        }
    });
}

// [Helper] 오버레이 창 띄우기 (기존 기능 유지)
fn trigger_overlay<R: Runtime>(app_handle: &AppHandle<R>) {
    if let Some(window) = app_handle.get_webview_window("overlay") {
        if !window.is_visible().unwrap_or(false) {
             window.show().ok();
             window.set_focus().ok();
        }
    } else {
        WebviewWindowBuilder::new(
            app_handle,
            "overlay",
            WebviewUrl::App("overlay.html".into())
        )
        .fullscreen(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .build().ok();
    }
}