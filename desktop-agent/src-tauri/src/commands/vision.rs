use active_win_pos_rs::get_active_window;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::command;

#[cfg(target_os = "windows")]
use std::ffi::OsString;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStringExt;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveWindowInfo {
    pub timestamp_ms: u64,
    pub title: String,
    pub process_path: String,
    pub app_name: String,
    pub window_id: String,
    pub process_id: u64,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub fn _get_active_window_info_internal() -> Result<ActiveWindowInfo, String> {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_millis() as u64;

    match get_active_window() {
        Ok(active_window) => {
            let app_name = Path::new(&active_window.process_path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
                .unwrap_or(active_window.app_name);

            Ok(ActiveWindowInfo {
                timestamp_ms,
                title: active_window.title,
                process_path: active_window.process_path.to_string_lossy().into_owned(),
                app_name: app_name,
                window_id: active_window.window_id,
                process_id: active_window.process_id,
                x: active_window.position.x,
                y: active_window.position.y,
                width: active_window.position.width,
                height: active_window.position.height,
            })
        }
        Err(e) => Err(format!("Failed to get active window info: {:?}", e)),
    }
}

#[command]
pub fn get_current_active_window_info() -> Result<ActiveWindowInfo, String> {
    _get_active_window_info_internal()
}

const MIN_VISIBLE_WIDTH: i32 = 120;
const MIN_VISIBLE_HEIGHT: i32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub title: String,
    pub app_name: String,
    pub is_visible_on_screen: bool,
    pub rect: WinRect,
}

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
    "Settings", 
];

#[cfg(target_os = "windows")]
fn get_process_path_from_pid(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }

    unsafe {
        let handle_result = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);

        if let Ok(handle) = handle_result {
            let mut buffer = [0u16; MAX_PATH as usize];
            let mut size = MAX_PATH;

            let success = QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_FORMAT(0),
                PWSTR(buffer.as_mut_ptr()),
                &mut size,
            );

            let _ = windows::Win32::Foundation::CloseHandle(handle);

            if success.is_ok() {
                return OsString::from_wide(&buffer[..size as usize])
                    .into_string()
                    .ok();
            }
        }
        None
    }
}

#[cfg(target_os = "windows")]
struct EnumContext {
    windows: Vec<WindowInfo>,
    foreground_hwnd: HWND,
    covered_rgn: HRGN,
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let context = &mut *(lparam.0 as *mut EnumContext);

    if IsIconic(hwnd).as_bool() {
        return TRUE;
    }

    let has_owner = match GetWindow(hwnd, GW_OWNER) {
        Ok(handle) => handle.0 != std::ptr::null_mut(),
        Err(_) => false,
    };

    if IsWindowVisible(hwnd).as_bool() && !has_owner {
        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        if width < MIN_VISIBLE_WIDTH || height < MIN_VISIBLE_HEIGHT {
            return TRUE;
        }

        let length = GetWindowTextLengthW(hwnd);
        if length > 0 {
            let mut buffer = vec![0u16; (length + 1) as usize];
            let copied_len = GetWindowTextW(hwnd, &mut buffer);

            if copied_len > 0 {
                let title_raw = OsString::from_wide(&buffer[..copied_len as usize]);
                if let Ok(title) = title_raw.into_string() {
                    let trimmed_title = title.trim();

                    if IGNORED_TITLES.contains(&trimmed_title) {
                        return TRUE;
                    }

                    let mut pid: u32 = 0;
                    GetWindowThreadProcessId(hwnd, Some(&mut pid));

                    let mut app_name = String::from("Unknown");
                    let mut is_system = false;

                    if let Some(path) = get_process_path_from_pid(pid) {
                        let p = path.to_lowercase();
                        is_system = WINDOWS_SYSTEM_PATHS
                            .iter()
                            .any(|sys| p.starts_with(&sys.to_lowercase()));

                        if !is_system {
                            if let Some(name) = Path::new(&path).file_name() {
                                app_name = name.to_string_lossy().into_owned();
                            }
                        }
                    }

                    if !is_system {
                        let current_win_rgn = CreateRectRgnIndirect(&rect);
                        let visible_part_rgn = CreateRectRgn(0, 0, 0, 0);

                        let region_type = CombineRgn(
                            visible_part_rgn,
                            current_win_rgn,
                            context.covered_rgn,
                            RGN_DIFF, 
                        );

                        let mut is_visually_visible = false;

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

                            CombineRgn(
                                context.covered_rgn,
                                context.covered_rgn,
                                current_win_rgn,
                                RGN_OR, 
                            );
                        }

                        DeleteObject(HGDIOBJ(current_win_rgn.0));
                        DeleteObject(HGDIOBJ(visible_part_rgn.0));
                    }
                }
            }
        }
    }
    TRUE
}

pub fn _get_all_visible_windows_internal() -> Vec<WindowInfo> {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let foreground_hwnd = GetForegroundWindow();
            let covered_rgn = CreateRectRgn(0, 0, 0, 0);

            let mut context = EnumContext {
                windows: Vec::new(),
                foreground_hwnd,
                covered_rgn,
            };

            let lparam = LPARAM(&mut context as *mut _ as isize);
            let _ = EnumWindows(Some(enum_window_callback), lparam);

            DeleteObject(HGDIOBJ(context.covered_rgn.0));

            context.windows
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        vec![("Unsupported OS".to_string(), false)]
    }
}

#[command]
pub fn get_visible_windows() -> Result<Vec<WindowInfo>, String> {
    let windows = _get_all_visible_windows_internal();
    Ok(windows)
}

pub fn extract_semantic_keywords(app_name: &str, window_title: &str) -> Vec<String> {
    let full_text = format!("{} {}", app_name, window_title).to_lowercase();
    
    full_text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

pub fn get_semantic_tokens(app_name: &str, window_title: &str) -> Vec<String> {
    extract_semantic_keywords(app_name, window_title)
}
