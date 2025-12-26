// 파일 위치: Force-Focus/desktop-agent/src-tauri/src/logging.rs

use crate::commands;
use chrono::{DateTime, Datelike, Local, Timelike}; // 시간 및 날짜 처리
use serde::{Deserialize, Serialize}; // JSON 직렬화/역직렬화
use serde_json;
use std::env; // 환경 변수 접근
use std::fs::{self, File, OpenOptions}; // 파일 시스템 작업
use std::io::prelude::*; // 파일 쓰기
use std::path::PathBuf; // 경로 관리
use std::thread; // 백그라운드 스레드 생성
use std::time::Duration;
use tauri::AppHandle;

use crate::commands::InputStatsArcMutex;

// 로그에 저장할 데이터 형식 정의 ---
// 주기적으로 수집한 정보 담는 구조체
#[derive(Debug, Serialize, Deserialize)] // Debug, Serialize, Deserialize 트레이트 파생
pub struct ActivityLogEntry {
    pub timestamp: String, // ISO 8601 형식의 시간 (예: "2023-10-27T10:00:00+09:00")
    pub active_window: Option<commands::ActiveWindowInfo>, // 활성 창 정보 (없을 수도 있으므로 Option)
    pub input_stats: Option<commands::InputStats>, // 입력 통계 정보 (없을 수도 있으므로 Option)

                                                   // 참고: 프로세스 요약은 데이터가 너무 커질 수 있으므로, 초기 단계에서는 포함 안함
                                                   // 필요시 나중에 추가하거나, 요약된 형태로 저장하는 것을 고려
                                                   // pub processes_summary: Option<Vec<commands::ProcessSummary>>,
}

// 로그 파일을 저장할 기본 디렉토리를 반환
// 애플리케이션 데이터 디렉토리 찾기
pub fn get_log_dir() -> Result<PathBuf, String> {
    // OS별 기본 경로 설정
    let mut base_path: PathBuf = if cfg!(target_os = "windows") {
        // Windows: %APPDATA% (Roaming)
        env::var("APPDATA")
            .map_err(|e| format!("APPDATA environment variable not found: {}", e))?
            .into()
    } else {
        // 그 외 OS: HOME 디렉토리 사용
        env::var("HOME")
            .map_err(|e| format!("HOME environment variable not found: {}", e))?
            .into()
    };

    // 애플리케이션 고유의 서브디렉토리 추가
    base_path.push("Force-Focus");
    base_path.push("logs");

    Ok(base_path)
}

// 특정 날짜의 로그 파일 경로를 반환 (예: 2023-10-27.jsonl)
pub fn get_log_file_path(log_dir: &PathBuf, date: &DateTime<Local>) -> PathBuf {
    let file_name = format!("{}.jsonl", date.format("%Y-%m-%d")); // JSON Lines 형식
    log_dir.join(file_name)
}

// 주기적으로 데이터를 수집하고 파일에 로깅하는 함수
pub fn start_data_collection_and_logging(
    input_stats_arc_mutex: InputStatsArcMutex,
    interval_secs: u64,
) {
    let log_dir_result = get_log_dir(); // 로그 디렉토리 경로 가져오기

    if let Err(e) = &log_dir_result {
        eprintln!("Failed to get log directory: {}", e);
        return; // 디렉토리 경로를 얻지 못하면 함수 종료
    }

    let log_dir = log_dir_result.unwrap();

    // 로그 디렉토리가 없으면 생성
    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory {:?}: {}", log_dir, e);
        return; // 디렉토리 생성 실패 시 함수 종료
    }

    // 백그라운드 스레드에서 데이터 수집 및 로깅을 수행
    thread::spawn(move || {
        loop {
            // 무한 루프
            let current_time: DateTime<Local> = Local::now();
            let log_file_path = get_log_file_path(&log_dir, &current_time); // 현재 날짜의 로그 파일 경로

            // 로깅할 엔트리 초기화
            let mut log_entry = ActivityLogEntry {
                timestamp: current_time.to_rfc3339(), // ISO 8601 형식의 현재 시간
                active_window: None,
                input_stats: None,
            };

            // 1. 활성 창 정보 수집
            match commands::_get_active_window_info_internal() {
                Ok(active_window_info) => {
                    log_entry.active_window = Some(active_window_info);
                }
                Err(e) => eprintln!("Logging: Failed to get active window info: {}", e),
            }

            // 2. 입력 통계 수집 (InputStatsArcMutex에서 직접 읽어오기)
            if let Ok(stats_guard) = input_stats_arc_mutex.lock() {
                log_entry.input_stats = Some(stats_guard.clone());
            } else {
                eprintln!("Logging: Failed to lock input_stats_arc_mutex");
            }

            // 3. 로그 엔트리 파일에 기록 (JSON Lines 형식)
            if let Ok(json_line) = serde_json::to_string(&log_entry) {
                match OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_file_path)
                {
                    Ok(mut file) => {
                        if let Err(e) = writeln!(file, "{}", json_line) {
                            eprintln!("Failed to write to log file {:?}: {}", log_file_path, e);
                        } else {
                            // println!("Logged data to {:?}", log_file_path);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to open log file {:?}: {}", log_file_path, e);
                    }
                }
            } else {
                eprintln!("Failed to serialize log entry to JSON");
            }

            // 다음 로깅까지 대기
            thread::sleep(Duration::from_secs(interval_secs));
        }
    });
}
