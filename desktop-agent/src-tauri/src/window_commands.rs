use tauri::{AppHandle, Manager, Runtime, State};
use std::sync::Mutex;
use crate::app_core::AppCore;

/// 'overlay' 창을 숨기고, FSM 상태를 리셋하는 Tauri 커맨드
#[tauri::command]
pub fn hide_overlay<R: Runtime>(
    app_handle: AppHandle<R>,
    state: State<Mutex<AppCore>>, // [New] AppCore 주입
) -> Result<(), String> {
    
    // 1. [Logic] FSM 상태 강제 리셋 (이게 없으면 1초 뒤에 창이 또 뜸!)
    {
        let mut app = state.lock().map_err(|_| "Failed to lock AppCore")?;
        app.state_engine.manual_reset();
        println!("GUI: Overlay hidden, State reset.");
    }

    // 2. [View] 물리적인 창 숨기기 (기존 코드 유지)
    if let Some(overlay_window) = app_handle.get_webview_window("overlay") {
        if let Err(e) = overlay_window.hide() {
            let err_msg = format!("Failed to hide overlay window: {:?}", e);
            eprintln!("{}", err_msg);
            return Err(err_msg);
        }
        Ok(())
    } else {
        // 창이 없어도 로직은 리셋되었으므로 성공으로 간주하거나 로그만 출력
        let err_msg = "Overlay window not found during hide request.".to_string();
        eprintln!("{}", err_msg);
        // Err(err_msg) // 굳이 에러를 낼 필요는 없음
        Ok(())
    }
}