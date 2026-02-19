// 파일 위치: Force-Focus/desktop-agent/src-tauri/src/commands.rs

/*
새로운 데이터를 추가하는 방법
1. InputStats 구조체에 새 필드 추가
2. 수집 로직 추가 (input_monitor.rs)
3. to_activity_vector_json 함수에 키/값 을 추가

*/

use active_win_pos_rs::get_active_window; // 활성 창 정보를 가져오는 함수
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH}; // 타임스탬프 생성을 위해 필요
use tauri::{command, State}; // active-win-pos-rs::ActiveWindow 구조체 필드에 PathBuf가 포함

use std::sync::{Arc, Mutex};
use sysinfo::System;

use rdev::{listen, Event, EventType};
use std::thread;

use std::path::Path;

// [추가] Windows API 사용을 위한 모듈 import (Windows 환경에서만 컴파일)
// [변경] windows 크레이트 import
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, BOOL, FALSE, HANDLE, HWND, LPARAM, MAX_PATH, RECT, TRUE,
};
use windows::Win32::Graphics::Gdi::{
    CombineRgn, CreateRectRgn, CreateRectRgnIndirect, DeleteObject, GetRgnBox, HGDIOBJ, HRGN,
    NULLREGION, RGN_COMBINE_MODE, RGN_DIFF, RGN_OR,
};

use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindow, GetWindowRect, GetWindowTextLengthW,
    GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible, GW_OWNER,
};
use crate::app_core::AppCore;
use crate::model_update_manager::ModelUpdateManager;

#[cfg(target_os = "windows")]
use std::ffi::OsString;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStringExt;

// --- 공유 상태 관리 ---
// Mutex<System>만 포함하며, System 인스턴스를 공유 상태로 관리합니다.
pub struct SysinfoState(pub Mutex<System>);

// 사용자 입력 통계 추적을 위한 공유 상태
// 앱 시작 시 한 번 초기화되어 계속 사용되므로 Arc로 공유됩니다.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InputStats {
    // 키/클릭/휠 이벤트만 카운트
    pub meaningful_input_events: u64,
    // 키/클릭/휠의 마지막 타임스탬프
    pub last_meaningful_input_timestamp_ms: u64,

    // 마우스 이동 전용 타임스탬프
    pub last_mouse_move_timestamp_ms: u64,

    // 모니터링 시작 시점
    pub start_monitoring_timestamp_ms: u64,

    // 시각 센서 데이터 (보이는 창 목록)
    // app_core.rs에서 _get_all_visible_windows_internal() 결과를 여기에 채워 넣습니다.
    #[serde(default)]
    pub visible_windows: Vec<WindowInfo>,
}

// FastAPI 모델 activity_vector
impl InputStats {
    /// 자신을 FastAPI가 요구하는 Dict[str, float]의 JSON 문자열로 변환
    pub fn to_activity_vector_json(&self) -> String {
        // serde_json::json! 매크로를 사용하여 Dict 생성
        let vector = serde_json::json!({
            "meaningful_input_events": self.meaningful_input_events,
            "last_meaningful_input_timestamp_ms": self.last_meaningful_input_timestamp_ms,
            "last_mouse_move_timestamp_ms": self.last_mouse_move_timestamp_ms,


            "visible_windows": self.visible_windows,

        });
        vector.to_string() // JSON 문자열로 반환
    }
}

pub type InputStatsArcMutex = Arc<Mutex<InputStats>>;

// --- 1. 활성 창 정보 관련 데이터 모델 및 명령어 ---

// 활성 창의 상세 정보를 담을 Rust 구조체
// 이 구조체는 웹 프론트엔드로 전송될 것이므로 Serialize/Deserialize 트레이트를 파생
// active_win_pos_rs::ActiveWindow 구조체와 유사하게 정의하되, 필요한 추가 필드를 포함
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveWindowInfo {
    pub timestamp_ms: u64,    // 정보 수집 시점의 타임스탬프 (밀리초)
    pub title: String,        // 창의 제목 (예: "Google Chrome - Wikipedia")
    pub process_path: String, // 실행 파일의 전체 경로 (예: "C:\Program Files\Google\Chrome\Application\chrome.exe")
    pub app_name: String,     // 애플리케이션 이름 (예: "chrome", "firefox")
    pub window_id: String,    // 운영체제별 고유 창 ID
    pub process_id: u64,      // 프로세스 ID
    pub x: f64,               // 창의 X 좌표
    pub y: f64,               // 창의 Y 좌표
    pub width: f64,           // 창의 너비
    pub height: f64,          // 창의 높이
}

// ActiveWindowInfo를 생성하는 내부 헬퍼 함수
pub fn _get_active_window_info_internal() -> Result<ActiveWindowInfo, String> {
    // 현재 시간을 밀리초 단위의 Unix 타임스탬프로 가져옴
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0)) // 에러 처리 추가
        .as_millis() as u64;

    // 현재 활성 창 정보
    match get_active_window() {
        Ok(active_window) => {
            // [수정] Task 1.1-Fix: app_name을 '실행 파일명'으로 통일
            // active_win_pos_rs는 "Google Chrome" 같은 이름을 줄 수 있지만,
            // 시각 센서(get_visible_windows)는 "chrome.exe"를 줍니다.
            // 데이터 일관성을 위해 process_path에서 파일명을 다시 추출합니다.
            let app_name = Path::new(&active_window.process_path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
                .unwrap_or(active_window.app_name); // 실패 시 기존 값(Friendly Name) 사용

            Ok(ActiveWindowInfo {
                timestamp_ms,
                title: active_window.title,
                process_path: active_window.process_path.to_string_lossy().into_owned(), // PathBuf를 String으로 변환
                app_name: app_name,
                window_id: active_window.window_id,
                process_id: active_window.process_id,
                x: active_window.position.x,
                y: active_window.position.y,
                width: active_window.position.width,
                height: active_window.position.height,
            })
        }
        // 활성 창을 가져오는 데 실패했을 경우 (에러나 활성 창 없음)
        Err(e) => Err(format!("Failed to get active window info: {:?}", e)),
    }
}

// 현재 활성 창의 정보를 가져오는 Tauri Command
#[tauri::command]
pub fn get_current_active_window_info() -> Result<ActiveWindowInfo, String> {
    _get_active_window_info_internal()
}

// --- 2. 시스템 상태 관련 데이터 모델 및 명령어 ---

// 모든 프로세스에 대한 요약 정보를 담을 Rust 구조체
// 이 구조체는 웹 프론트엔드로 전송될 것이므로 Serialize/Deserialize 트레이트를 파생합니다.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessSummary {
    pub name: String,           // 프로세스의 이름 (예: "chrome", "notepad.exe")
    pub start_time_unix_s: u64, // 프로세스 시작 시점의 Unix 타임스탬프 (초 단위)
}

// 시스템의 모든 실행 중인 프로세스 요약 정보를 가져오는 Tauri Command
#[command]
pub fn get_all_processes_summary(
    sys_state: State<'_, SysinfoState>,
) -> Result<Vec<ProcessSummary>, String> {
    // SysinfoState가 Mutex<System>만 가지므로 sys_state.0.lock()으로 접근합니다.
    let mut sys_guard = sys_state.0.lock().unwrap();

    // 시스템 정보 새로 고침
    // sysinfo::System::refresh_all()은 프로세스 목록을 포함한 대부분의 시스템 정보를 갱신합니다.
    sys_guard.refresh_all();

    let mut processes_summary = Vec::new();
    // sys_guard.processes()는 (Pid, &Process) 형태의 Iterator를 반환합니다.
    for (_pid, process) in sys_guard.processes() {
        if (process.start_time() > 0) {
            processes_summary.push(ProcessSummary {
                name: process.name().to_string_lossy().into_owned(), // &OsStr을 String으로 안전하게 변환
                start_time_unix_s: process.start_time(),
            });
        }
    }
    Ok(processes_summary)
}

// --- 3. (향후 추가될) 스크린샷 관련 데이터 모델 및 명령어 ---
// (현재 비어 있음)

// --- 4. 사용자 입력 및 유휴 시간 관련 데이터 모델 및 명령어 ---

// 현재까지의 사용자 입력 빈도 통계를 반환하는 Command
#[command]
pub fn get_input_frequency_stats(
    input_stats_arc_mutex: State<'_, InputStatsArcMutex>,
) -> Result<InputStats, String> {
    // input_stats_arc_mutex는 직접 Arc<Mutex<InputStats>>의 참조 가짐.
    // .lock().unwrap()을 호출하여 MutexGuard를 얻고, 내부 데이터를 클론
    let stats = input_stats_arc_mutex.lock().unwrap();
    Ok((*stats).clone())
}

// --- 5. 시각 센서 (Visible Windows) ---
// 화면에 보이는 창을 수집

// --- [설정] 시각적 임계값 ---
// 이 크기보다 작게 보이는 창(자투리)은 '안 보임' 처리합니다.
const MIN_VISIBLE_WIDTH: i32 = 120;
const MIN_VISIBLE_HEIGHT: i32 = 100;

// [1] Debug 구현을 위한 Rust용 Rect 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

// [2] WindowInfo 구조체 정의
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub title: String,
    // 시맨틱 태깅을 위한 앱 이름 필드
    // (예: chrome, Code)
    pub app_name: String,

    pub is_visible_on_screen: bool,

    pub rect: WinRect,
}

// --- OS 유틸리티 경로 목록 (필터링용) ---
#[cfg(target_os = "windows")]
const WINDOWS_SYSTEM_PATHS: &[&str] = &[
    "C:\\WINDOWS\\SYSTEM32",
    "C:\\WINDOWS\\SYSTEMAPPS",
    "C:\\PROGRAM FILES\\WINDOWSAPPS",
    "C:\\WINDOWS\\EXPLORER.EXE",
];

#[cfg(target_os = "windows")]
const IGNORED_TITLES: &[&str] = &[
    "Shell Handwriting Canvas",
    "Microsoft Text Input Application",
    "Program Manager",
    "Settings", // 윈도우 설정 같은 백그라운드 앱
];

// --- PID로 프로세스 경로를 얻는 헬퍼 함수 ---
#[cfg(target_os = "windows")]
fn get_process_path_from_pid(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }

    unsafe {
        // OpenProcess는 Result<HANDLE>을 반환합니다.
        let handle_result = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);

        if let Ok(handle) = handle_result {
            // drop되면 자동으로 닫히지 않는 Raw Handle일 수 있으므로 CloseHandle 필요
            // (windows-rs의 OwnedHandle을 쓰지 않고 Raw 호출 시)

            let mut buffer = [0u16; MAX_PATH as usize];
            let mut size = MAX_PATH;

            // K32QueryFullProcessImageNameW 사용 (Kernel32 wrapper)
            let success = QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_FORMAT(0), // 0: Win32 format
                PWSTR(buffer.as_mut_ptr()),
                &mut size,
            );

            let _ = windows::Win32::Foundation::CloseHandle(handle);

            if success.is_ok() {
                // 슬라이스에서 문자열 변환
                return OsString::from_wide(&buffer[..size as usize])
                    .into_string()
                    .ok();
            }
        }
        None
    }
}

// --- EnumWindows를 위한 상태 구조체 ---
#[cfg(target_os = "windows")]
struct EnumContext {
    windows: Vec<WindowInfo>,
    foreground_hwnd: HWND,
    // 지금까지 화면을 덮어버린 영역들의 합집합 (누적 가림막)
    covered_rgn: HRGN,
}

// --- 콜백 함수  ---
#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let context = &mut *(lparam.0 as *mut EnumContext);

    // 1. 상태 체크 (IsIconic, IsWindowVisible 반환값은 BOOL/bool)
    if IsIconic(hwnd).as_bool() {
        return TRUE;
    }

    // [수정됨] GetWindow 결과 처리 로직
    // GetWindow(GW_OWNER)가 Ok(handle)을 반환하면 소유자가 있다는 뜻(즉, 팝업/자식 창).
    // Err를 반환하면 소유자가 없다는 뜻(즉, 최상위 창).
    let has_owner = match GetWindow(hwnd, GW_OWNER) {
        Ok(handle) => handle.0 != std::ptr::null_mut(), // 핸들이 0이 아니면 소유자가 있음
        Err(_) => false,                                // 에러(NULL)면 소유자가 없음
    };

    // 소유자가 없고(최상위 창이고) + 보이는 창인 경우에만 처리
    if IsWindowVisible(hwnd).as_bool() && !has_owner {
        // 2. 좌표 가져오기
        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        // 크기 1차 필터링
        if width < MIN_VISIBLE_WIDTH || height < MIN_VISIBLE_HEIGHT {
            return TRUE;
        }

        let length = GetWindowTextLengthW(hwnd);
        if length > 0 {
            let mut buffer = vec![0u16; (length + 1) as usize];
            // GetWindowTextW는 복사된 문자 수를 반환
            let copied_len = GetWindowTextW(hwnd, &mut buffer);

            if copied_len > 0 {
                let title_raw = OsString::from_wide(&buffer[..copied_len as usize]);
                if let Ok(title) = title_raw.into_string() {
                    let trimmed_title = title.trim();

                    // 블랙리스트 체크
                    if IGNORED_TITLES.contains(&trimmed_title) {
                        return TRUE;
                    }

                    let mut pid: u32 = 0;
                    GetWindowThreadProcessId(hwnd, Some(&mut pid));

                    // [수정] app_name 추출을 위한 변수
                    let mut app_name = String::from("Unknown");
                    let mut is_system = false;

                    if let Some(path) = get_process_path_from_pid(pid) {
                        let p = path.to_lowercase();
                        is_system = WINDOWS_SYSTEM_PATHS
                            .iter()
                            .any(|sys| p.starts_with(&sys.to_lowercase()));

                        // 경로에서 파일명(app_name) 추출
                        if !is_system {
                            if let Some(name) = Path::new(&path).file_name() {
                                app_name = name.to_string_lossy().into_owned();
                            }
                        }
                    }

                    if !is_system {
                        // ---------------------------------------------------------
                        // [GDI Region Logic - windows-rs 버전]
                        // ---------------------------------------------------------

                        // A. 현재 창 Region 생성
                        let current_win_rgn = CreateRectRgnIndirect(&rect);

                        // B. Visible Region (현재 - 누적)
                        let visible_part_rgn = CreateRectRgn(0, 0, 0, 0);

                        // CombineRgn: 리턴값은 Region Type (i32)
                        let region_type = CombineRgn(
                            visible_part_rgn,
                            current_win_rgn,
                            context.covered_rgn,
                            RGN_DIFF, // RGN_COMBINE_MODE(4) -> DIFF
                        );

                        let mut is_visually_visible = false;

                        // NULLREGION == 1 (windows-rs 상수에 따라 다를 수 있으니 상수 사용 권장)
                        if region_type
                            != windows::Win32::Graphics::Gdi::GDI_REGION_TYPE(NULLREGION.0 as i32)
                        {
                            let mut box_rect = RECT::default();
                            GetRgnBox(visible_part_rgn, &mut box_rect);

                            let visible_w = box_rect.right - box_rect.left;
                            let visible_h = box_rect.bottom - box_rect.top;

                            if visible_w >= MIN_VISIBLE_WIDTH && visible_h >= MIN_VISIBLE_HEIGHT {
                                is_visually_visible = true;
                            }
                        }

                        if is_visually_visible {
                            context.windows.push(WindowInfo {
                                app_name,
                                title: trimmed_title.to_string(),
                                is_visible_on_screen: true,
                                rect: WinRect {
                                    left: rect.left,
                                    top: rect.top,
                                    right: rect.right,
                                    bottom: rect.bottom,
                                },
                            });

                            // D. 누적(Union)
                            CombineRgn(
                                context.covered_rgn,
                                context.covered_rgn,
                                current_win_rgn,
                                RGN_OR, // RGN_COMBINE_MODE(2) -> OR
                            );
                        } else {
                            // // 안 보이는 창 (디버깅용 포함)
                            // context.windows.push(WindowInfo {
                            //     title: trimmed_title.to_string(),
                            //     is_visible_on_screen: false,
                            //     rect: WinRect {
                            //         left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom,
                            //     },
                            // });
                        }

                        // E. 리소스 해제 (HGDIOBJ 변환 필요)
                        // windows-rs의 HRGN은 HGDIOBJ로 바로 cast 되지 않을 수 있음.
                        // 핸들 값(.0)을 이용해 HGDIOBJ 생성
                        DeleteObject(HGDIOBJ(current_win_rgn.0));
                        DeleteObject(HGDIOBJ(visible_part_rgn.0));
                    }
                }
            }
        }
    }
    TRUE
}

/// [내부 함수] 현재 화면에 보이는 모든 창의 제목을 수집합니다.
/// (Windows 전용 구현)
pub fn _get_all_visible_windows_internal() -> Vec<WindowInfo> {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let foreground_hwnd = GetForegroundWindow();
            // [초기화] 빈 영역(0,0,0,0) 생성
            let covered_rgn = CreateRectRgn(0, 0, 0, 0);

            // 콜백에 전달할 상태 객체 생성
            let mut context = EnumContext {
                windows: Vec::new(),
                foreground_hwnd,
                covered_rgn,
            };

            // EnumWindows 호출 (안정적인 순회)
            let lparam = LPARAM(&mut context as *mut _ as isize);
            let _ = EnumWindows(Some(enum_window_callback), lparam);

            // [정리] 다 쓰고 난 누적 영역 삭제
            // [핵심 수정] 'as *mut _' 제거 및 HGDIOBJ로 올바르게 감싸기
            // HRGN의 내부 값(.0)을 꺼내 HGDIOBJ 생성자에 전달합니다.
            DeleteObject(HGDIOBJ(context.covered_rgn.0));

            context.windows
        }
    }

    // (비-Windows 환경을 위한 더미 구현)
    #[cfg(not(target_os = "windows"))]
    {
        vec![("Unsupported OS".to_string(), false)]
    }
}

#[tauri::command]
pub fn get_system_stats(state: State<Mutex<AppCore>>) -> Result<serde_json::Value, String> {
    let app = state.lock().map_err(|_| "Failed to lock AppCore")?;

    let stats = serde_json::json!({
        "current_state": format!("{:?}", app.state_engine.get_state_string()), 
        "gauge_ratio": app.state_engine.get_gauge_ratio(),
        // 게이지가 꽉 찼으면 오버레이 활성화
        // (Threshold 값은 state_engine 상수를 참고하거나 1.0 기준)
        "is_overlay_active": app.state_engine.get_gauge_ratio() >= 1.0 
    });

    Ok(stats)
}

/// [Tauri 커맨드] 프론트엔드나 app_core에서 호출 가능한 래퍼
#[command]
pub fn get_visible_windows() -> Result<Vec<WindowInfo>, String> {
    let windows = _get_all_visible_windows_internal();
    Ok(windows)
}

// --- 토큰화 및 숫자 필터링 = 시맨틱 태깅 ---
// 목표: ML 모델과 동일한 'Simple Tokenization' (Spec: ML_models.md)
// 1. App Name + Window Title 결합
// 2. Non-alphanumeric 기준 분리
// 3. 소문자 변환
pub fn extract_semantic_keywords(app_name: &str, window_title: &str) -> Vec<String> {
    let full_text = format!("{} {}", app_name, window_title).to_lowercase();
    
    full_text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

// [Wrapper] 기존 코드 호환성 유지
pub fn get_semantic_tokens(app_name: &str, window_title: &str) -> Vec<String> {
    extract_semantic_keywords(app_name, window_title)
}

// ================================================================
// ML 모델 수동 업데이트 트리거
// ================================================================

/// 프론트엔드 UI에서 "업데이트 확인" 버튼 클릭 시 호출
/// - token: 프론트엔드 AuthContext의 JWT 토큰
/// - manager: main.rs에서 .manage()로 등록된 ModelUpdateManager 인스턴스 (자동 주입)
#[tauri::command]
pub async fn check_model_update(
    token: String,
    manager: State<'_, ModelUpdateManager>, 
) -> Result<bool, String> {
    println!("🖱️ [Command] Manual update requested.");
    
    // 비동기 작업 수행 (UI 스레드 차단 방지)
    // check_and_update는 Result<bool, String>을 반환하므로 그대로 사용 가능
    manager.check_and_update(&token).await
}