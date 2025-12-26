use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};

///  트레이 메뉴 생성 및 이벤트 핸들러 설정
pub fn setup_tray_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    // 1. 메뉴 아이템 생성
    let show_hide_i = MenuItem::with_id(app, "toggle", "열기/숨기기 (Show/Hide)", true, None::<&str>)?;
    // [추가] 세션 종료 메뉴 (옵션)
    let end_session_i = MenuItem::with_id(app, "end_session", "세션 강제 종료 (End Session)", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "앱 완전 종료 (Quit)", true, None::<&str>)?;
    
    // 2. 메뉴 구성
    let menu = Menu::with_items(app, &[&show_hide_i, &end_session_i, &quit_i])?;

    // 3. 트레이 아이콘 생성
    let _tray = TrayIconBuilder::with_id("tray")
        .icon(app.default_window_icon().unwrap().clone()) // 앱 기본 아이콘 사용
        .menu(&menu)
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "quit" => {
                    println!("Tray: Quit clicked");
                    app.exit(0); // 앱 완전 종료
                }
                "toggle" => {
                    println!("Tray: Toggle clicked");
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            window.hide().unwrap();
                        } else {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                }
                "end_session" => {
                    println!("Tray: End Session clicked");
                    // [참고] Rust 내부에서 커맨드를 직접 호출하는 것은 복잡할 수 있음.
                    // 여기서는 단순히 이벤트를 보내 프론트엔드가 처리하게 하거나,
                    // backend_communicator의 로직을 직접 호출하는 별도 함수를 만들어야 함.
                    // (임시로 로그만 출력)
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
             // (선택) 트레이 아이콘 좌클릭 시 메인 창 토글
             if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                 let app = tray.app_handle();
                 if let Some(window) = app.get_webview_window("main") {
                     if window.is_visible().unwrap_or(false) {
                         let _ = window.hide();
                     } else {
                         let _ = window.show();
                         let _ = window.set_focus();
                     }
                 }
             }
        })
        .build(app)?;

    Ok(())
}