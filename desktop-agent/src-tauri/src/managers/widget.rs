// [추가] Task 4.10: '위젯' 로직을 lib.rs에서 분리 (관심사 분리)
use crate::SessionStateArcMutex; // lib.rs에서 정의한 전역 세션 상태
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Runtime, Url, WebviewUrl, WebviewWindowBuilder, WindowEvent};

/// [추가] setup 훅에서 호출될 '위젯' 이벤트 리스너 설정 함수
pub fn setup_widget_listeners<R: Runtime>(
    app_handle: AppHandle<R>,
    session_state_mutex: SessionStateArcMutex,
) {
    let main_window = app_handle.get_webview_window("main").unwrap();
    let app_handle_clone = app_handle.clone(); // 스레드간 이동

    // 패닉 방지: 억지로 시간을 빼지 않고 Option으로 '초기 상태'를 표현
    // None = 아직 한 번도 포커스를 얻은 적 없음
    let last_focus_gain_time: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
    let last_focus_gain_time_clone = last_focus_gain_time.clone();

    main_window.on_window_event(move |event| {
        match event {
            // [유지] v2 API: '메인 창'이 '포커스를 잃음' (최소화 또는 다른 창 클릭)
            WindowEvent::Focused(false) => {
                let session_state = session_state_mutex.lock().unwrap();

                // 쿨다운 체크: 포커스를 얻은 지 200ms도 안 지났는데 잃었다면? -> 무시 (복원 노이즈)
                let last_gain_opt = *last_focus_gain_time_clone.lock().unwrap();

                if let Some(last_gain) = last_gain_opt {
                    // 포커스를 얻은 기록이 있다면, 경과 시간 체크
                    if last_gain.elapsed() < Duration::from_millis(200) {
                        println!("Ignored Focused(false) due to cooldown (restore noise).");
                        return;
                    }
                }
                // None인 경우(앱 켜고 처음)는 쿨다운 없이 통과 (즉시 위젯 표시 가능)

                if session_state.is_some() {
                    // 세션이 '활성' 상태일 때만 '위젯'을 띄움
                    println!("Main window lost focus, showing widget...");
                    show_widget_window(&app_handle_clone);
                }
            }

            // [유지] v2 API: '메인 창'이 '포커스를 얻음'
            WindowEvent::Focused(true) => {
                // 포커스 획득 시점 기록 (Some으로 감싸서 저장)
                let mut last_gain = last_focus_gain_time_clone.lock().unwrap();
                *last_gain = Some(Instant::now());

                // '메인 창'이 보이므로 '위젯'을 숨김
                if let Some(widget) = app_handle_clone.get_webview_window("widget") {
                    // [개선] 이미 숨겨져 있다면 호출하지 않음 (불필요한 연산 방지)
                    if widget.is_visible().unwrap_or(false) {
                        widget.hide().ok();
                    }
                }
            }
            _ => {}
        }
    });
}

/// [추가] '위젯'을 띄우는 'Get-or-Create' 헬퍼 함수
fn show_widget_window<R: Runtime>(app_handle: &AppHandle<R>) {
    if let Some(widget_window) = app_handle.get_webview_window("widget") {
        // [개선] 이미 보인다면 show() 호출 안 함 (포커스 뺏기 방지)
        if !widget_window.is_visible().unwrap_or(false) {
            widget_window.show().ok();
        }
    } else {
        println!("Widget window not found. Re-creating...");

        #[cfg(debug_assertions)]
        let url = WebviewUrl::External("http://localhost:1420/widget.html".parse().unwrap());
        #[cfg(not(debug_assertions))]
        let url = WebviewUrl::App("widget.html".into());

        // [Fix] 화면 해상도에 따른 동적 위치 계산
        // 기본값 (FHD 기준 우측 상단)
        let mut pos_x = 1680.0;
        let mut pos_y = 20.0;
        
        let width = 220.0;
        let margin = 20.0;

        if let Ok(Some(monitor)) = app_handle.primary_monitor() {
            let size = monitor.size(); // PhysicalSize
            let scale_factor = monitor.scale_factor(); // f64
            
            // Physical -> Logical 변환
            let screen_logical_width = size.width as f64 / scale_factor;
            
            // 우측 상단 좌표 계산: (화면 너비 - 위젯 너비 - 마진)
            pos_x = screen_logical_width - width - margin;
            pos_y = margin;
            
            println!("Calculated Widget Position: ({}, {}) for Screen Width: {}", pos_x, pos_y, screen_logical_width);
        } else {
             println!("Failed to detect monitor. Using default position.");
        }

        if let Err(e) = WebviewWindowBuilder::new(app_handle, "widget", url)
            .always_on_top(true)
            .decorations(false)
            .resizable(false)
            .skip_taskbar(true)
            .inner_size(width, 70.0)
            .position(pos_x, pos_y) // [Modified] 동적 좌표 적용
            .visible(true)
            .build()
        {
            eprintln!("Failed to re-create widget window: {:?}", e);
        }
    }
}
