use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

///  트레이 메뉴 생성 및 이벤트 핸들러 설정
pub fn setup_tray_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    // 1. 메뉴 아이템
    let toggle_i = MenuItem::with_id(app, "toggle", "열기 / 숨기기 (Toggle)", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "종료 (Quit)", true, None::<&str>)?;
    
    // 2. 메뉴 구성
    let menu = Menu::with_items(app, &[&toggle_i, &quit_i])?;

    // 클릭 쿨다운(Term) 관리를 위한 상태
    // 초기값은 과거 시간으로 설정하여 첫 클릭이 바로 되도록 함
    let last_click = Arc::new(Mutex::new(Instant::now() - Duration::from_secs(1)));
    let last_click_clone = last_click.clone();

    // 3. 트레이 아이콘 생성
    let _tray = TrayIconBuilder::with_id("tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .menu_on_left_click(false) // 왼쪽 클릭 시 메뉴가 뜨지 않도록 설정 (우클릭에만 반응)
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "quit" => {
                    println!("Tray: Quit clicked");
                    app.exit(0);
                }
                "toggle" => {
                    // 메뉴에서 클릭했을 때의 동작
                    if let Some(window) = app.get_webview_window("main") {
                        let is_visible = window.is_visible().unwrap_or(false);
                        if is_visible {
                            let _ = window.hide();
                        } else {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
                _ => {}
            }
        })
        .on_tray_icon_event(move |tray, event| {
             // 아이콘 좌클릭 시 토글 로직
             if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                 let app = tray.app_handle();
                 
                 // 쿨다운 체크
                 let mut last = last_click_clone.lock().unwrap();
                 if last.elapsed() < Duration::from_millis(200) {
                     return; 
                 }
                 *last = Instant::now();

                 if let Some(window) = app.get_webview_window("main") {
                     let is_visible = window.is_visible().unwrap_or(false);
                     
                     if is_visible {
                         let _ = window.hide();
                     } else {
                         let _ = window.unminimize();
                         let _ = window.show();
                         let _ = window.set_focus();
                     }
                 }
             }
        })
        .build(app)?;

    Ok(())
}
