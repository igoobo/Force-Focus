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

// InputStatsArcMutex 타입은 lib.rs에서 단일 정의됨 (C-5 해결)

#[command]
pub fn get_input_frequency_stats(
    input_stats_arc_mutex: State<'_, crate::InputStatsArcMutex>,
) -> Result<InputStats, String> {
    let stats = input_stats_arc_mutex.lock().map_err(|_| "Failed to lock InputStats".to_string())?;
    Ok((*stats).clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_activity_vector_json_serialization() {
        let stats = InputStats {
            meaningful_input_events: 150,
            last_meaningful_input_timestamp_ms: 1000000,
            last_mouse_move_timestamp_ms: 1000500,
            start_monitoring_timestamp_ms: 900000,
            visible_windows: vec![
                WindowInfo {
                    app_name: "Code".to_string(),
                    title: "main.rs - VSCode".to_string(),
                    is_visible_on_screen: true,
                    rect: crate::commands::vision::WinRect { 
                        left: 0, top: 0, right: 1920, bottom: 1080 
                    },
                }
            ],
        };

        let json_str = stats.to_activity_vector_json();
        
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Should produce valid JSON");
        
        assert_eq!(parsed["meaningful_input_events"], 150);
        assert_eq!(parsed["last_meaningful_input_timestamp_ms"], 1000000);
        assert_eq!(parsed["last_mouse_move_timestamp_ms"], 1000500);
        
        let windows = parsed["visible_windows"].as_array().expect("visible_windows should be an array");
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0]["app_name"], "Code");
        assert_eq!(windows[0]["title"], "main.rs - VSCode");
        
        assert!(parsed.get("start_monitoring_timestamp_ms").is_none(), "start_monitoring_timestamp_ms should not be serialized into the activity vector");
    }
}
