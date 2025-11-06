use tauri::{AppHandle, Manager, Runtime}; 

/// 'overlay' 창을 숨기는 Tauri 커맨드
/// React의 InterventionOverlay.tsx에서 이 커맨드를 호출
#[tauri::command]
pub fn hide_overlay<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
    // "overlay" 레이블을 가진 창
    if let Some(overlay_window) = app_handle.get_webview_window("overlay") {
        if let Err(e) = overlay_window.hide() {
            let err_msg = format!("Failed to hide overlay window: {:?}", e);
            eprintln!("{}", err_msg);
            return Err(err_msg);
        }
        Ok(())
    } else {
        let err_msg = "Overlay window not found during hide request.".to_string();
        eprintln!("{}", err_msg);
        Err(err_msg)
    }
}