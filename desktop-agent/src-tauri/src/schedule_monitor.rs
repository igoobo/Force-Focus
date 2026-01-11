use tauri::{AppHandle, Emitter, Manager};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use std::process::Command;
use chrono::{Local, Timelike, Datelike}; // 시간 계산용

use crate::{StorageManagerArcMutex, SessionStateArcMutex, ActiveSessionInfo};
use tauri_plugin_notification::NotificationExt; // 알림 플러그인

/// 스케줄 모니터링 루프 시작
pub fn start_monitor_loop(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        println!("Schedule Monitor: Started background loop (Interval: 60s)");

        loop {
            // 정각(00초)에 가깝게 실행되도록 보정할 수 있으나, 
            // 일단 단순하게 60초 간격으로 체크
            sleep(Duration::from_secs(60)).await;

            if let Err(e) = check_and_execute_schedules(&app_handle).await {
                eprintln!("Schedule Monitor Error: {}", e);
            }
        }
    });
}

/// 스케줄 확인 및 실행 로직
async fn check_and_execute_schedules(app: &AppHandle) -> Result<(), String> {
    // 1. LSN 접근
    let storage_state = app.try_state::<StorageManagerArcMutex>()
        .ok_or("StorageManager state not found")?;

    // 2. 현재 로그인한 사용자 ID 확인 (격리)
    let user_id = {
        let storage = storage_state.lock().map_err(|e| e.to_string())?;
        
        // load_auth_token 반환값: Option<(Access, Refresh, Email, UserID)>
        match storage.load_auth_token()? {
            Some((_, _, _, uid)) => uid, // user_id 추출
            None => return Ok(()), // 로그인 정보가 없으면 스케줄 실행 안 함 (오프라인)
        }
    };

    // 3. '내 ID'로 등록된 활성 스케줄만 조회
    let schedules = {
        let storage = storage_state.lock().map_err(|e| e.to_string())?;
        storage.get_active_schedules(&user_id)? 
    };

    if schedules.is_empty() {
        return Ok(());
    }

    // 4. 현재 시간 확인 (Local Time 기준)
    let now = Local::now();
    let current_weekday = now.weekday().num_days_from_monday() as u8; // 0=Mon ~ 6=Sun
    let current_time_str = now.format("%H:%M").to_string(); // "14:00" 형식

    for schedule in schedules {
        // [조건 1] 요일 일치 여부
        if !schedule.days_of_week.contains(&current_weekday) {
            continue;
        }

        // [조건 2] 시작 시간 일치 여부 (HH:MM 비교)
        // DB에는 "HH:MM:SS"로 저장되어 있으므로 앞 5글자만 비교
        if !schedule.start_time.starts_with(&current_time_str) {
            continue;
        }

        println!("Schedule Monitor: Matched schedule '{}' (ID: {})", schedule.name, schedule.id);

        // [실행] 스케줄 트리거
        trigger_schedule(app, &schedule, &storage_state).await?;
    }

    Ok(())
}

async fn trigger_schedule(
    app: &AppHandle, 
    schedule: &crate::storage_manager::LocalSchedule, 
    storage_state: &StorageManagerArcMutex
) -> Result<(), String> {
    // A. 이미 세션이 진행 중인지 확인 (중복 실행 방지)
    let session_state = app.try_state::<SessionStateArcMutex>()
        .ok_or("SessionState not found")?;
    
    {
        let session = session_state.lock().map_err(|e| e.to_string())?;
        if session.is_some() {
            println!("Schedule Monitor: Session is already active. Skipping auto-start.");
            return Ok(());
        }
    }

    // B. 연결된 Task 정보 조회 (실행 파일 + 인자)
    let (target_executable, target_arguments) = if let Some(task_id) = &schedule.task_id {
        let storage = storage_state.lock().map_err(|e| e.to_string())?;
        if let Some(task) = storage.get_task_by_id(task_id)? {
             (task.target_executable, task.target_arguments)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // C. [RPA] 프로그램 강제 실행
    if let Some(exe_path) = target_executable {
        if !exe_path.is_empty() {
            println!("Schedule Monitor: Launching program -> {}", exe_path);
            
            let mut cmd = Command::new(&exe_path);
            
            // 인자가 있으면 공백 기준으로 분리하여 추가
            if let Some(args_str) = target_arguments {
                if !args_str.is_empty() {
                    // 단순 공백 분리 (복잡한 escaping은 추후 고려)
                    let args: Vec<&str> = args_str.split_whitespace().collect();
                    cmd.args(args);
                }
            }

            // Rust Command를 사용하여 외부 프로세스 실행 (비동기 spawn)
            // 주의: 경로나 권한 문제로 실패할 수 있음 (에러 로그만 남김)
            match Command::new(&exe_path).spawn() {
                Ok(_) => println!("Schedule Monitor: Program launched successfully."),
                Err(e) => eprintln!("Schedule Monitor: Failed to launch '{}': {}", exe_path, e),
            }
        }
    }

    // D. 세션 시작 (로컬 상태 업데이트)
    let new_session = ActiveSessionInfo {
        session_id: format!("auto-{}", uuid::Uuid::new_v4()),
        task_id: schedule.task_id.clone(),
        start_time_s: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };

    {
        let mut session = session_state.lock().map_err(|e| e.to_string())?;
        let storage = storage_state.lock().map_err(|e| e.to_string())?;
        
        storage.save_active_session(&new_session)?;
        *session = Some(new_session.clone());
    }

    // E. 알림 및 UI 업데이트
    // 프론트엔드에 세션 시작 이벤트 전송 (MainView 전환용)
    // [주의] 이 이벤트 리스너가 MainView에 있어야 함
    app.emit("session-started", &new_session).map_err(|e| e.to_string())?;
    
    // OS 네이티브 알림
    let _ = app.notification()
        .builder()
        .title("집중 스케줄 시작")
        .body(format!("'{}' 스케줄에 따라 집중 모드를 시작합니다.", schedule.name))
        .show();

    Ok(())
}