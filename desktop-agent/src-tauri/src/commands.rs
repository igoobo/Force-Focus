// 파일 위치: Force-Focus/desktop-agent/src-tauri/src/commands.rs

use tauri::{command, State};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH}; // 타임스탬프 생성을 위해 필요
use active_win_pos_rs::get_active_window; // 활성 창 정보를 가져오는 함수
use std::path::PathBuf; // active-win-pos-rs::ActiveWindow 구조체 필드에 PathBuf가 포함

use sysinfo::{System};
use std::sync::{Mutex, Arc};

use rdev::{listen, Event, EventType};
use std::thread;




// --- 공유 상태 관리 ---
// Mutex<System>만 포함하며, System 인스턴스를 공유 상태로 관리합니다.
pub struct SysinfoState(pub Mutex<System>);

// 사용자 입력 통계 추적을 위한 공유 상태
// 앱 시작 시 한 번 초기화되어 계속 사용되므로 Arc로 공유됩니다.
#[derive(Debug, Default, Clone, Serialize, Deserialize)] 
pub struct InputStats {
    pub total_input_events: u64, // 총 입력 이벤트 수
    pub last_input_timestamp_ms: u64, // 마지막 입력 이벤트 발생 시점 (밀리초)
    pub start_monitoring_timestamp_ms: u64, // 모니터링 시작 시점 (밀리초)
}

pub type InputStatsArcMutex = Arc<Mutex<InputStats>>;


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
}


// ActiveWindowInfo를 생성하는 내부 헬퍼 함수
pub fn _get_active_window_info_internal() -> Result<ActiveWindowInfo, String> {
    // 현재 시간을 밀리초 단위의 Unix 타임스탬프로 가져옴
    let timestamp_ms = SystemTime::now().duration_since(UNIX_EPOCH)
                                    .unwrap_or_else(|_| std::time::Duration::from_secs(0)) // 에러 처리 추가
                                    .as_millis() as u64;

    // 현재 활성 창 정보
    match get_active_window() {
        Ok(active_window) => {
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
            })
        },
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
    pub name: String,            // 프로세스의 이름 (예: "chrome", "notepad.exe")
    pub start_time_unix_s: u64,  // 프로세스 시작 시점의 Unix 타임스탬프 (초 단위)

}


// 시스템의 모든 실행 중인 프로세스 요약 정보를 가져오는 Tauri Command
#[command]
pub fn get_all_processes_summary(sys_state: State<'_, SysinfoState>) -> Result<Vec<ProcessSummary>, String> {
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
pub fn get_input_frequency_stats(input_stats_arc_mutex: State<'_, InputStatsArcMutex>) -> Result<InputStats, String> {
    // input_stats_arc_mutex는 직접 Arc<Mutex<InputStats>>의 참조 가짐.
    // .lock().unwrap()을 호출하여 MutexGuard를 얻고, 내부 데이터를 클론
    let stats = input_stats_arc_mutex.lock().unwrap();
    Ok((*stats).clone())
}