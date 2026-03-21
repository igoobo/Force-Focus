use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use sysinfo::System;
use tauri::{command, State};
use crate::core::app::AppCore;

// --- 공유 상태 관리 ---
pub struct SysinfoState(pub Mutex<System>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessSummary {
    pub name: String,
    pub start_time_unix_s: u64,
}

#[command]
pub fn get_all_processes_summary(
    sys_state: State<'_, SysinfoState>,
) -> Result<Vec<ProcessSummary>, String> {
    let mut sys_guard = sys_state.0.lock().map_err(|_| "Failed to lock SysinfoState".to_string())?;
    sys_guard.refresh_all();
    let mut processes_summary = Vec::new();
    for (_pid, process) in sys_guard.processes() {
        if process.start_time() > 0 {
            processes_summary.push(ProcessSummary {
                name: process.name().to_string_lossy().into_owned(),
                start_time_unix_s: process.start_time(),
            });
        }
    }
    Ok(processes_summary)
}

#[tauri::command]
pub fn get_system_stats(state: State<Mutex<AppCore>>) -> Result<serde_json::Value, String> {
    let app = state.lock().map_err(|_| "Failed to lock AppCore")?;
    let stats = serde_json::json!({
        "current_state": format!("{:?}", app.state_engine.get_state_string()), 
        "gauge_ratio": app.state_engine.get_gauge_ratio(),
        "is_overlay_active": app.state_engine.get_gauge_ratio() >= 1.0 
    });
    Ok(stats)
}
