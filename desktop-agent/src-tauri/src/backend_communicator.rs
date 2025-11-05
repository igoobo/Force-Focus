// 파일 위치: src-tauri/src/backend_communicator.rs

use reqwest::Client;
use serde::Serialize;
use tauri::{command, State};

// --- 1. 상수 정의 ---

// MSW 목업이 아닌, 실제 FastAPI 백엔드 서버의 주소
// (로컬 개발 환경을 가정)
const API_BASE_URL: &str = "http://127.0.0.1:8000/api/v1"; 

// '가상 사용자' 전략을 위한 하드코딩된 인증 토큰
// msw/handlers.ts에 정의된 'mock-jwt-token-123'
const MOCK_AUTH_TOKEN: &str = "mock-jwt-token-123";

// --- 2. API 요청/응답을 위한 구조체 ---

/// `POST /feedback` API의 Request Body 스키마
/// (API 명세서: event_id, feedback_type)
#[derive(Debug, Serialize)]
struct FeedbackRequest<'a> {
    event_id: &'a str, // 이벤트 ID (임시)
    feedback_type: &'a str, // "is_work" 또는 "distraction_ignored"
}

// (API 응답을 위한 Deserialize 구조체도 필요시 추가)
// #[derive(Debug, Deserialize)]
// struct FeedbackResponse { ... }

// --- 3. BackendCommunicator 상태 정의 ---

/// reqwest::Client를 전역 상태로 관리하기 위한 구조체
/// Client는 내부에 Arc를 가지고 있어 복제(clone)에 저렴
pub struct BackendCommunicator {
    client: Client,
}

impl BackendCommunicator {
    /// 앱 시작 시 호출될 생성자
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

// --- 4. 이 모듈에 속한 Tauri 커맨드 정의 ---

/// '개입'에 대한 사용자 피드백을 서버로 전송하는 비동기(async) 커맨드
///
/// # Arguments
/// * `feedback_type` - 프론트엔드에서 받은 피드백 (예: "is_work")
/// * `comm_state` - Tauri가 주입하는 BackendCommunicator 전역 상태
#[command]
pub async fn submit_feedback(
    feedback_type: String, 
    comm_state: State<'_, BackendCommunicator>
) -> Result<(), String> {
    
    // (임시) 현재는 event_id가 없으므로 임의의 값을 사용
    let event_id = "temp_event_001"; 

    let request_body = FeedbackRequest {
        event_id,
        feedback_type: &feedback_type,
    };

    let url = format!("{}/feedback", API_BASE_URL);

    println!( // 디버깅 로그
        "Submitting feedback to {}: type={}",
        url, request_body.feedback_type
    );

    match comm_state.client
        .post(&url)
        .bearer_auth(MOCK_AUTH_TOKEN) // '가상 사용자' 토큰 사용
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                println!("Feedback submitted successfully.");
                Ok(())
            } else {
                let error_msg = format!("API Error: {}", response.status());
                eprintln!("{}", error_msg);
                Err(error_msg)
            }
        }
        Err(e) => {
            let error_msg = format!("Reqwest Error: {}", e);
            eprintln!("{}", error_msg);
            Err(error_msg)
        }
    }
}