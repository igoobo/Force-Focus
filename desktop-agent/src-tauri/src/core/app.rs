// 파일 위치: src-tauri/src/app_core.rs

use crate::{
    commands::{self, vision::{ActiveWindowInfo, WindowInfo}},
    core::state::{self, StateEngine, InterventionTrigger},
    commands::input::InputStatsArcMutex,
    SessionStateArcMutex,
    StorageManagerArcMutex,
    ai::inference::InferenceEngine
};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};
use tauri::path::BaseDirectory;
use uuid::Uuid;
use std::collections::{HashMap, VecDeque};

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
    pub last_inference_result: crate::ai::inference::InferenceResult,

    // 현재 모니터링 중인 이벤트의 ID (피드백 연결용)
    pub current_event_id: Option<String>,

    // 런타임에 로드되는 글로벌 맵 캐시
    pub global_map: HashMap<String, f64>,

    // X_burstiness 계산을 위한 최근 12틱(1분) delta_input 큐
    pub delta_history: VecDeque<f64>,
}

impl AppCore {
    pub fn new<R: Runtime>(app_handle: &AppHandle<R>) -> Self {
        // 1. 쓰기 가능한 AppData 폴더 경로 확보 (예: C:\Users\User\AppData\Roaming\com.forcefocus.app\models)
        let app_data_dir = app_handle.path().app_data_dir().expect("Failed to get AppData directory");
        let model_dir = app_data_dir.join("models");

        if !model_dir.exists() {
            std::fs::create_dir_all(&model_dir).unwrap();
        }

        let model_path = model_dir.join("personal_model.onnx");
        let scaler_path = model_dir.join("scaler_params.json");
        let map_path = model_dir.join("global_map.json");

        // 1. 번들 리소스 경로 해석
        let bundled_model = app_handle.path().resolve("resources/models/personal_model.onnx", BaseDirectory::Resource).ok();
        let bundled_scaler = app_handle.path().resolve("resources/models/scaler_params.json", BaseDirectory::Resource).ok();
        let bundled_map = app_handle.path().resolve("resources/models/global_map.json", BaseDirectory::Resource).ok();

        // 2. [핵심 해결] 개발 모드(Debug)에서는 무조건 덮어쓰기, 배포 모드(Release)에서는 없을 때만 복사
        #[cfg(debug_assertions)]
        {
            println!("🛠️ [Dev Mode] Forcing overwrite of ML artifacts to ensure latest base model.");
            if let Some(src) = &bundled_model { let _ = std::fs::copy(src, &model_path); }
            if let Some(src) = &bundled_scaler { let _ = std::fs::copy(src, &scaler_path); }
            if let Some(src) = &bundled_map { let _ = std::fs::copy(src, &map_path); }
        }

        #[cfg(not(debug_assertions))]
        {
            if !model_path.exists() {
                if let Some(src) = &bundled_model { let _ = std::fs::copy(src, &model_path); }
            }
            if !scaler_path.exists() {
                if let Some(src) = &bundled_scaler { let _ = std::fs::copy(src, &scaler_path); }
            }
            if !map_path.exists() {
                if let Some(src) = &bundled_map { let _ = std::fs::copy(src, &map_path); }
            }
        }

        // 3. 글로벌 맵 로딩
        let global_map: std::collections::HashMap<String, f64> = if let Ok(data) = std::fs::read_to_string(&map_path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

        // 4. ML 엔진 로드
        let inference_engine = match InferenceEngine::new(
            model_path.to_str().unwrap_or(""), 
            scaler_path.to_str().unwrap_or("")
        ) {
            Ok(engine) => Some(engine),
            Err(e) => {
                eprintln!("⚠️ [AppCore] ML Model load failed: {}", e);
                None 
            }
        };

        Self {
            inference_engine,
            state_engine: StateEngine::new(),
            last_event_count: 0,
            last_inference_result: crate::ai::inference::InferenceResult::Inlier,
            current_event_id: None,
            global_map,
            delta_history: VecDeque::with_capacity(12),
        }
    }

    // 동적 로드된 맵을 기반으로 점수 계산 (Spec: Simple Tokenization & Exact Match)
    fn calculate_context_score(&self, app_name: &str, title: &str) -> f64 {
        let full_text = format!("{} {}", app_name, title).to_lowercase();
        
        let mut score = 0.0;
        let mut count = 0.0;
        let mut found = false;
        
        // Split by non-alphanumeric (Spec-compliant)
        let tokens: Vec<&str> = full_text.split(|c: char| !c.is_alphanumeric()).collect();

        for token in tokens {
            if token.is_empty() { continue; }

            // Exact Match Lookup
            if let Some(&val) = self.global_map.get(token) {
                score += val;
                count += 1.0;
                found = true;
            }
        }
        
        if !found { return 0.0; } // Neutral (Unknown) - Spec says 0.0
        if count == 0.0 { return 0.0; }
        
        score / count
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
                    if let Ok(window_info) = crate::commands::vision::_get_active_window_info_internal() {
                        
                        // 시각 센서 (Visible Windows) 수집
                        let mut visible_windows_raw = commands::vision::_get_all_visible_windows_internal();

                        // 시맨틱 태깅 (Semantic Tagging)
                        // 원본 제목을 '토큰화 + 숫자 필터링'된 문자열로 세탁
                        for window in &mut visible_windows_raw {
                            let tokens = commands::vision::get_semantic_tokens(&window.app_name, &window.title);
                            if !tokens.is_empty() {
                                window.title = tokens.join(" ");
                            } else {
                                window.title = String::new(); // 개인정보 보호
                            }
                        }

                        // 활성 창(Active Window) 태깅
                        // 활성 창 역시 동일한 로직으로 토큰을 추출합니다.
                        let active_tokens = commands::vision::get_semantic_tokens(&window_info.app_name, &window_info.title);
                        let sanitized_active_title = active_tokens.join(" ");

                        // UUID 생성 (Flag 발급)
                        let client_evt_id = format!("evt-{}", Uuid::new_v4());

                        // AppCore 상태에 ID 저장 (피드백 연결용)
                        core.current_event_id = Some(client_evt_id.clone());

                        // InputStats에 시각 데이터 업데이트
                        // [!] ML 모델을 위해 '전경 여부'도 포함할 수 있지만, 현재는 title만 저장
                        input_stats.visible_windows = visible_windows_raw;
                        // InputStats를 JSON 문자열로 직렬화 (commands.rs 헬퍼 호출)
                        let activity_vector_json = input_stats.to_activity_vector_json(); // LSN 저장용

                        // 2. ML Feature 생성 (Delta Event 등)
                        let raw_delta = current_events.saturating_sub(core.last_event_count);
                        core.last_event_count = current_events;
                        
                        // [핵심 해결] Feature Clipping (Winsorization)
                        // OS가 5초 동안 1000개의 마우스 이벤트를 뱉어내더라도, 
                        // 모델이 소화할 수 있는 최대 임계치(예: 50.0)로 값을 잘라냅니다.
                        // 50번 이상의 움직임은 어차피 "최고 수준의 몰입 상태"이므로 그 이상은 무의미합니다.
                        let delta_f64 = (raw_delta as f64).min(50.0); 

                        let silence_sec = if input_stats.last_meaningful_input_timestamp_ms > 0 {
                            (now_ms.saturating_sub(input_stats.last_meaningful_input_timestamp_ms) as f64) / 1000.0
                        } else { 0.0 };
                        
                        // 크롬의 경우 Context Score는 0.1 로 정상 계산됨
                        let context_score = core.calculate_context_score(&window_info.app_name, &window_info.title);

                        // [신규] train.py의 check_mouse_active 로직 완벽 동기화
                        // 0 <= (evt_ts - mouse_ts) <= 5.0 인 경우 1.0, 아니면 0.0
                        let mouse_delta_sec = if input_stats.last_mouse_move_timestamp_ms > 0 {
                            (now_ms.saturating_sub(input_stats.last_mouse_move_timestamp_ms) as f64) / 1000.0
                        } else {
                            f64::MAX // 마우스 입력이 한 번도 없었던 경우
                        };
                        
                        let x_mouse = if mouse_delta_sec >= 0.0 && mouse_delta_sec <= 5.0 { 
                            1.0 
                        } else { 
                            0.0 
                        };

                        // 2. 수학적 동기화
                        // 이제 delta_f64가 최대 50.0으로 제한되므로, X_log_input은 최대 ln(51) ≈ 3.93 을 넘지 못합니다.
                        let x_log_input = (delta_f64 + 1.0).ln();

                        // X_burstiness 역시 비정상적으로 튀지 않고 안정적인 표준편차를 유지합니다.
                        core.delta_history.push_back(delta_f64);
                        if core.delta_history.len() > 12 { core.delta_history.pop_front(); }
                        let n = core.delta_history.len() as f64;
                        let x_burstiness = if n > 1.0 {
                            let mean = core.delta_history.iter().sum::<f64>() / n;
                            let variance = core.delta_history.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
                            variance.sqrt()
                        } else { 0.0 };

                        let sig_x = 1.0 / (delta_f64 + 0.1);
                        let sigmoid = 1.0 / (1.0 + (-sig_x as f64).exp());
                        let x_interaction = sigmoid * context_score;

                        // 3. 완벽히 일치하는 ML 벡터 구성
                        let ml_vector = [
                            context_score, 
                            x_log_input, 
                            silence_sec,
                            x_burstiness,
                            x_mouse,
                            x_interaction 
                        ];


                        // 4. 데이터 저장 (학습용 데이터셋 구축)
                        // LSN에 이벤트를 저장해야 나중에 꺼내서 학습할 수 있습니다.
                        let storage = storage_manager_mutex.lock().unwrap();
                        let raw_json = serde_json::json!({
                            "delta_events": raw_delta,
                            "silence_sec": silence_sec,
                            "window_title": window_info.title, // 원본 제목 (학습용)
                            "ml_vector": ml_vector
                        }).to_string();

                        storage
                            .cache_event(
                                &active_session.session_id,
                                &client_evt_id,
                                &window_info.app_name,
                                &sanitized_active_title,
                                &activity_vector_json, // JSON 문자열 전달
                            )
                            .unwrap_or_else(|e| eprintln!("Failed to cache event: {}", e));
                        drop(storage);

                        // 5. ML 추론 (모델이 준비된 경우)
                        if let Some(engine) = &mut core.inference_engine {
                            // Pass active_tokens (Vec<String>) for proper cache lookup
                            match engine.infer(ml_vector, active_tokens.clone()) {
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
                        println!("🔔 [Action] Notification (Click-Through)");
                        
                        // 1. 창이 없으면 생성
                        ensure_overlay_exists(&app_handle_clone);

                        if let Some(overlay_window) = app_handle_clone.get_webview_window("overlay") {
                            // 1. 투명 모드(Click-Through) 활성화
                            let _ = commands::window::set_overlay_ignore_cursor_events(app_handle_clone.clone(), true);
                            
                            // 2. 창 표시
                            let _ = commands::window::show_overlay(app_handle_clone.clone());
                            
                            // 3. [핵심 수정] 특정 윈도우에 직접 발송
                            // 문자열 대신 확실한 JSON 형태 전송 권장하지만, 기존 호환성을 위해 문자열 유지하되 타겟팅 변경
                            overlay_window.emit("intervention-trigger", "notification").ok();
                        }
                    },
                    InterventionTrigger::TriggerOverlay => {
                        println!("🚫 [Action] Blocking Overlay");
                        
                        // 1. 창이 없으면 생성
                        ensure_overlay_exists(&app_handle_clone);

                        if let Some(overlay_window) = app_handle_clone.get_webview_window("overlay") {
                            // 1. 차단 모드(Block Input) 활성화
                            let _ = commands::window::set_overlay_ignore_cursor_events(app_handle_clone.clone(), false);
                            
                            // 2. 창 표시
                            let _ = commands::window::show_overlay(app_handle_clone.clone());
                            
                            // 3. [핵심 수정] 특정 윈도우에 직접 발송
                            println!("➡️ Sending 'overlay' event directly to window...");
                            overlay_window.emit("intervention-trigger", "overlay").ok();
                            
                            // [안전장치] 혹시 React가 렌더링 중이라 못 받을까봐 100ms 뒤 한 번 더 쏠 수도 있음 (선택 사항)
                            // std::thread::spawn(move || {
                            //     std::thread::sleep(Duration::from_millis(200));
                            //     overlay_window.emit("intervention-trigger", "overlay").ok();
                            // });
                        }
                    },
                    InterventionTrigger::DoNothing => {
                        // [Fix] 게이지가 줄어들어 FOCUS 상태(30 미만)로 돌아오면 오버레이 숨김
                        // (기존에는 0.0일 때만 숨겨서 29초여도 오버레이가 안 꺼지는 문제 발생)
                        let should_hide = core.state_engine.get_gauge_ratio() <= 0.0 
                            || core.state_engine.get_state() == crate::core::state::FSMState::FOCUS;

                        if should_hide {
                             if let Some(window) = app_handle_clone.get_webview_window("overlay") {
                                 if window.is_visible().unwrap_or(false) {
                                     // [Fix] Deadlock 방지: commands::window::hide_overlay 호출 대신 직접 로직 수행
                                     // (commands::window::hide_overlay는 AppCore lock을 다시 시도하므로 교착상태 발생)
                                     
                                     // 1. 상태 리셋 (제거함: 자연스러운 회복에서는 게이지를 0으로 초기화하면 안 됨)
                                     // core.state_engine.manual_reset(); 
                                     
                                     // 2. 창 숨기기
                                     let _ = window.hide();
                                     let _ = window.set_ignore_cursor_events(false);
                                     println!("GUI: Overlay hidden (Natural Recovery), Gauge preserved.");
                                 }
                             }
                        }
                    }
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

// [Helper] 오버레이 생성 도우미 (표시는 show_overlay에 위임)
fn ensure_overlay_exists<R: Runtime>(app_handle: &AppHandle<R>) {
    if app_handle.get_webview_window("overlay").is_none() {
        // 투명 창 속성으로 생성
        WebviewWindowBuilder::new(
            app_handle,
            "overlay",
            WebviewUrl::App("overlay.html".into())
        )
        .fullscreen(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .transparent(true)  // [필수]
        .decorations(false) // [필수]
        .visible(false)     // 일단 숨김 상태로 생성
        .build().ok();
    }
}