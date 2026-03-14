use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{command, State};
use crate::commands::vision::WindowInfo;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InputStats {
    pub meaningful_input_events: u64,
    pub last_meaningful_input_timestamp_ms: u64,
    pub last_mouse_move_timestamp_ms: u64,
    pub start_monitoring_timestamp_ms: u64,
    #[serde(default)]
    pub visible_windows: Vec<WindowInfo>,
}

impl InputStats {
    pub fn to_activity_vector_json(&self) -> String {
        let vector = serde_json::json!({
            "meaningful_input_events": self.meaningful_input_events,
            "last_meaningful_input_timestamp_ms": self.last_meaningful_input_timestamp_ms,
            "last_mouse_move_timestamp_ms": self.last_mouse_move_timestamp_ms,
            "visible_windows": self.visible_windows,
        });
        vector.to_string()
    }
}

pub type InputStatsArcMutex = Arc<Mutex<InputStats>>;

#[command]
pub fn get_input_frequency_stats(
    input_stats_arc_mutex: State<'_, InputStatsArcMutex>,
) -> Result<InputStats, String> {
    let stats = input_stats_arc_mutex.lock().unwrap();
    Ok((*stats).clone())
}
