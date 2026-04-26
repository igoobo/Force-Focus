// 파일 위치: src-tauri/src/commands/auth.rs
// backend_comm.rs에서 분리된 인증 관련 Tauri 커맨드 (U-3 해결)

use tauri::{command, State};
use crate::StorageManagerArcMutex;

/// 로그인 커맨드
#[command]
pub fn login(
    access_token: String,
    refresh_token: String,
    user_email: String,
    user_id: String,
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
) -> Result<(), String> {
    let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;
    storage_manager.save_auth_token(&access_token, &refresh_token, &user_email, &user_id)?;
    println!("User logged in: [REDACTED]");
    Ok(())
}

/// 로그아웃 커맨드
#[command]
pub fn logout(storage_manager_mutex: State<'_, StorageManagerArcMutex>) -> Result<(), String> {
    let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;
    storage_manager.delete_auth_token()?;
    println!("User logged out.");
    Ok(())
}

/// 앱 시작 시 로그인 상태 확인 (Auto-Login)
#[command]
pub fn check_auth_status(
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
) -> Result<Option<String>, String> {
    let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;
    let token_data = storage_manager.load_auth_token().map_err(|e| e.to_string())?;

    if let Some((_, _, email, _)) = token_data {
        println!("Auto-login: Found valid token for [REDACTED]");
        Ok(Some(email))
    } else {
        Ok(None)
    }
}
