// 파일 위치: Force-Focus/desktop-agent/src-tauri/src/commands.rs

use tauri::{command, State};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH}; // 타임스탬프 생성을 위해 필요
use active_win_pos_rs::get_active_window; // 활성 창 정보를 가져오는 함수
use std::path::PathBuf; // active-win-pos-rs::ActiveWindow 구조체 필드에 PathBuf가 포함


// --- 1. 활성 창 정보 관련 데이터 모델 및 명령어 ---

// 활성 창의 상세 정보를 담을 Rust 구조체
// 이 구조체는 웹 프론트엔드로 전송될 것이므로 Serialize/Deserialize 트레이트를 파생
// active_win_pos_rs::ActiveWindow 구조체와 유사하게 정의하되, 필요한 추가 필드를 포함
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveWindowInfo {
    pub timestamp_ms: u64, // 정보 수집 시점의 타임스탬프 (밀리초)
    pub title: String,     // 창의 제목 (예: "Google Chrome - Wikipedia")
    pub process_path: String, // 실행 파일의 전체 경로 (예: "C:\Program Files\Google\Chrome\Application\chrome.exe")
    pub app_name: String,  // 애플리케이션 이름 (예: "chrome", "firefox")
    pub window_id: String, // 운영체제별 고유 창 ID
    pub process_id: u64,   // 프로세스 ID
    pub x: f64,            // 창의 X 좌표
    pub y: f64,            // 창의 Y 좌표
    pub width: f64,        // 창의 너비
    pub height: f64,       // 창의 높이
    pub window_state: String, // 창의 상태 (예: "Focused", "Minimized", "Maximized" 등 - 이 값은 직접 판단해야 함)
}


// 현재 활성 창의 정보를 가져오는 Tauri Command
#[command]
pub fn get_current_active_window_info() -> Result<ActiveWindowInfo, String> {
    // 현재 시간을 밀리초 단위의 Unix 타임스탬프로 가져옵니다.
    let timestamp_ms = SystemTime::now().duration_since(UNIX_EPOCH)
                                .unwrap_or_else(|_| std::time::Duration::from_secs(0)) // 에러 처리 추가
                                .as_millis() as u64;

    // active-win-pos-rs 크레이트를 사용하여 현재 활성 창 정보를 가져옵니다.
    // get_active_window() 함수는 Result<ActiveWindow, ()>를 반환합니다.
    match get_active_window() {
        Ok(active_window) => {
            // https://docs.rs/active-win-pos-rs/latest/active_win_pos_rs/struct.ActiveWindow.html
            // active-win-pos-rs::ActiveWindow 구조체 필드 
            // pub title: String,
            // pub process_path: PathBuf,
            // pub app_name: String,
            // pub window_id: String,
            // pub process_id: u64,
            // pub position: WindowPosition { x: f64, y: f64, width: f64, height: f64 }

            // ActiveWindowInfo 구조체에 맞춰 데이터를 매핑합니다.
            Ok(ActiveWindowInfo {
                timestamp_ms,
                title: active_window.title,
                process_path: active_window.process_path.to_string_lossy().into_owned(), // PathBuf를 String으로 변환
                app_name: active_window.app_name,
                window_id: active_window.window_id,
                process_id: active_window.process_id,
                x: active_window.position.x,
                y: active_window.position.y,
                width: active_window.position.width,
                height: active_window.position.height,
                // window_state는 active-win-pos-rs에서 직접 제공하지 않으므로,
                // 현재로서는 "Focused"로 가정합니다. (나중에 더 정교한 로직 추가 가능)
                window_state: "Focused".to_string(),
            })
        },
        // 활성 창을 가져오는 데 실패했을 경우 (에러나 활성 창 없음)
        Err(()) => Err("Failed to get active window info.".to_string()),
    }
}

// --- 2. (향후 추가될) 시스템 상태 관련 데이터 모델 및 명령어 ---
// (현재 비어 있음)

// --- 3. (향후 추가될) 스크린샷 관련 데이터 모델 및 명령어 ---
// (현재 비어 있음)

// --- 4. (향후 추가될) 사용자 입력 및 유휴 시간 관련 데이터 모델 및 명령어 ---
// (현재 비어 있음)