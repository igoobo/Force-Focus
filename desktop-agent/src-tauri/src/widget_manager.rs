// [추가] Task 4.10: '위젯' 로직을 lib.rs에서 분리 (관심사 분리)
use tauri::{AppHandle, Manager, Runtime, WebviewWindowBuilder, WebviewUrl, Url, WindowEvent};
use crate::SessionStateArcMutex; // lib.rs에서 정의한 전역 세션 상태

/// [추가] setup 훅에서 호출될 '위젯' 이벤트 리스너 설정 함수
pub fn setup_widget_listeners<R: Runtime>(
    app_handle: AppHandle<R>,
    session_state_mutex: SessionStateArcMutex,
) {
    let main_window = app_handle.get_webview_window("main").unwrap();
    let app_handle_clone = app_handle.clone(); // 스레드간 이동

    main_window.on_window_event(move |event| {
        match event {
            // [수정] Task 4.11 (E0599): 'VisibilityChanged'(환각 API) -> 'Focused'(v2 API)
            // '메인 창'이 '포커스를 잃음' (최소화 또는 다른 창 클릭)
            WindowEvent::Focused(false) => {
                let session_state = session_state_mutex.lock().unwrap();
                if session_state.is_some() {
                    // 세션이 '활성' 상태일 때만 '위젯'을 띄움
                    println!("Main window lost focus, showing widget...");
                    show_widget_window(&app_handle_clone);
                }
            },
            
            // [수정] Task 4.11 (E0599): 'VisibilityChanged'(환각 API) -> 'Focused'(v2 API)
            // '메인 창'이 '포커스를 얻음'
            WindowEvent::Focused(true) => {
                // '메인 창'이 보이므로 '위젯'을 숨김
                if let Some(widget) = app_handle_clone.get_webview_window("widget") {
                    widget.hide().ok();
                }
            },
            _ => {}
        }
    });
}

/// [추가] '위젯'을 띄우는 'Get-or-Create' 헬퍼 함수
fn show_widget_window<R: Runtime>(app_handle: &AppHandle<R>) {
    if let Some(widget_window) = app_handle.get_webview_window("widget") {
        widget_window.show().ok();
    } else {
        println!("Widget window not found. Re-creating...");
        
        #[cfg(debug_assertions)]
        let url = WebviewUrl::External("http://localhost:1420/widget.html".parse().unwrap());
        #[cfg(not(debug_assertions))]
        let url = WebviewUrl::App("widget.html".into());

        if let Err(e) = WebviewWindowBuilder::new(app_handle, "widget", url)
            .always_on_top(true)
            .decorations(false)
            .resizable(false)
            .skip_taskbar(true)
            .inner_size(220.0, 70.0) 
            .position(1680.0, 20.0) // [임시]
            .visible(true)
            .build()
        {
            eprintln!("Failed to re-create widget window: {:?}", e);
        }
    }
}