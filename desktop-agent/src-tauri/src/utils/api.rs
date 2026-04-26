// 파일 위치: src-tauri/src/utils/api.rs
// backend_comm.rs에서 분리된 순수 네트워크 코어 (U-3 해결)

use reqwest::Client;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use anyhow::Result;

use dotenv::dotenv;
use std::env;

use crate::managers::storage::{CachedEvent, LocalSchedule, LocalTask};

// --- 1. 상수 정의 ---

pub fn get_api_base_url() -> String {
    const BUILD_TIME_URL: Option<&str> = option_env!("API_BASE_URL");
    
    if let Some(url) = BUILD_TIME_URL {
        return url.to_string();
    }

    dotenv().ok();
    env::var("API_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000/api/v1".to_string())
}

// --- 2. API 요청/응답 구조체 (DTO) ---

#[derive(Debug, Serialize, Clone)]
pub struct FeedbackPayload {
    pub client_event_id: String,
    pub feedback_type: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct SessionStartRequest<'a> {
    pub task_id: Option<&'a str>,
    pub goal_duration: u32,
}

#[derive(Debug, Deserialize)]
pub struct SessionStartResponse {
    pub session_id: String,
    pub start_time: String,
}

#[derive(Debug, Serialize)]
pub struct SessionEndRequest {
    pub user_evaluation_score: u8,
}

#[derive(Debug, Serialize)]
pub struct EventBatchRequest {
    pub events: Vec<EventData>,
}

#[derive(Debug, Serialize)]
pub struct EventData {
    pub session_id: String,
    pub client_event_id: String,
    pub timestamp: i64,
    pub app_name: String,
    pub window_title: String,
    pub activity_vector: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ApiTask {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub target_executable: Option<String>,
    pub target_arguments: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApiSchedule {
    pub id: String,
    pub user_id: String,
    pub task_id: Option<String>,
    pub name: String,
    pub start_time: String,
    pub end_time: String,
    pub days_of_week: Vec<u8>,
    pub start_date: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelVersionResponse {
    pub status: String,
    pub version: String,
    pub download_urls: ModelDownloadUrls,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelDownloadUrls {
    pub model: String,
    pub scaler: String,
}

// --- 3. BackendCommunicator ---

pub struct BackendCommunicator {
    pub client: Client,
}

impl BackendCommunicator {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    pub async fn check_latest_model_version(&self, token: &str) -> Result<ModelVersionResponse> {
        let url = format!("{}/desktop/models/latest", get_api_base_url());
        let resp = self.client.get(&url).bearer_auth(token).send().await?.error_for_status()?;
        let info: ModelVersionResponse = resp.json().await?;
        Ok(info)
    }

    pub async fn download_file(&self, endpoint: &str, save_path: &PathBuf, token: &str) -> Result<()> {
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!("{}{}", get_api_base_url(), endpoint)
        };
        let resp = self.client.get(&url).bearer_auth(token).send().await?.error_for_status()?;
        let mut file = File::create(save_path).await?;
        let mut stream = resp.bytes_stream();
        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        Ok(())
    }

    pub async fn send_feedback_batch(&self, feedbacks: Vec<FeedbackPayload>, token: &str) -> Result<(), String> {
        let url = format!("{}/desktop/feedback/batch", get_api_base_url());
        if feedbacks.is_empty() { return Ok(()); }
        let response = self.client.post(&url).bearer_auth(token).json(&feedbacks).send().await
            .map_err(|e| format!("Request failed: {}", e))?;
        if response.status().is_success() {
            println!("✅ Feedback batch sent successfully.");
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(format!("Server returned error {}: {}", status, text))
        }
    }

    pub async fn sync_events_batch(&self, events: Vec<CachedEvent>, token: &str) -> Result<(), String> {
        let url = format!("{}/events/batch", get_api_base_url());
        let event_data_list: Vec<EventData> = events.into_iter().filter_map(|e| {
            match serde_json::from_str(&e.activity_vector) {
                Ok(json_val) => Some(EventData {
                    session_id: e.session_id,
                    client_event_id: e.client_event_id,
                    timestamp: e.timestamp,
                    app_name: e.app_name,
                    window_title: e.window_title,
                    activity_vector: json_val,
                }),
                Err(err) => {
                    eprintln!("Failed to parse activity_vector JSON for event {}: {}", e.id, err);
                    None
                }
            }
        }).collect();
        if event_data_list.is_empty() { return Ok(()); }
        let request_body = EventBatchRequest { events: event_data_list };
        let response = self.client.post(&url).bearer_auth(token).json(&request_body).send().await
            .map_err(|e| format!("Request failed: {}", e))?;
        if response.status().is_success() {
            println!("Sync success!");
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(format!("Server returned error {}: {}", status, text))
        }
    }

    pub async fn fetch_tasks(&self, token: &str) -> Result<Vec<LocalTask>, String> {
        let url = format!("{}/desktop/data/tasks", get_api_base_url());
        let response = self.client.get(&url).bearer_auth(token).send().await
            .map_err(|e| format!("Failed to fetch tasks: {}", e))?;
        if response.status().is_success() {
            let api_tasks: Vec<ApiTask> = response.json().await.map_err(|e| format!("JSON parse error: {}", e))?;
            Ok(api_tasks.into_iter().map(|t| LocalTask {
                id: t.id, user_id: t.user_id, task_name: t.name, description: t.description,
                target_executable: t.target_executable, target_arguments: t.target_arguments, status: t.status,
            }).collect())
        } else {
            Err(format!("Server error (Tasks): {}", response.status()))
        }
    }

    pub async fn fetch_schedules(&self, token: &str) -> Result<Vec<LocalSchedule>, String> {
        let url = format!("{}/desktop/data/schedules", get_api_base_url());
        let response = self.client.get(&url).bearer_auth(token).send().await
            .map_err(|e| format!("Failed to fetch schedules: {}", e))?;
        if response.status().is_success() {
            let api_schedules: Vec<ApiSchedule> = response.json().await.map_err(|e| format!("JSON parse error: {}", e))?;
            Ok(api_schedules.into_iter().map(|s| LocalSchedule {
                id: s.id, user_id: s.user_id, task_id: s.task_id, name: s.name,
                start_time: s.start_time, end_time: s.end_time, days_of_week: s.days_of_week,
                start_date: s.start_date, is_active: s.is_active,
            }).collect())
        } else {
            Err(format!("Server error (Schedules): {}", response.status()))
        }
    }
}
