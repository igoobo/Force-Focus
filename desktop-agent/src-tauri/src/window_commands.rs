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

    // 2. 창 숨기기
    if let Some(overlay_window) = app_handle.get_webview_window("overlay") {
        overlay_window.hide().map_err(|e| e.to_string())?;
        // [안전장치] 숨길 때 차단 모드(false)로 복구해둬야 다음 번에 클릭 가능
        let _ = overlay_window.set_ignore_cursor_events(false); 
        Ok(())
    } else {
        Ok(())
    }
}
// ================================================================
// 점진적 개입을 위한 필수 제어 함수들
// ================================================================

/// 1. 오버레이 표시 (강제 최상단)
#[tauri::command]
pub fn show_overlay<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
    if let Some(overlay_window) = app_handle.get_webview_window("overlay") {
        overlay_window.show().map_err(|e| e.to_string())?;
        overlay_window.set_always_on_top(true).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Overlay window not found".to_string())
    }
}

/// 2. 마우스 클릭 투명화 제어 (핵심 기술)
/// - ignore: true  -> 마우스가 창을 뚫고 지나감 (투명 인간 모드 / 경고 단계)
/// - ignore: false -> 마우스가 창에 막힘 (차단 모드 / 개입 단계)
#[tauri::command]
pub fn set_overlay_ignore_cursor_events<R: Runtime>(
    app_handle: AppHandle<R>,
    ignore: bool,
) -> Result<(), String> {
    if let Some(overlay_window) = app_handle.get_webview_window("overlay") {
        // Tauri v2 API: set_ignore_cursor_events
        overlay_window.set_ignore_cursor_events(ignore)
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Overlay window not found".to_string())
    }
}