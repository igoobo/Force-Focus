// 파일 위치: src-tauri/src/state_engine.rs

use crate::commands::{ActiveWindowInfo, InputStats};
use std::time::{SystemTime, UNIX_EPOCH}; // commands.rs에서 정의한 데이터 모델

// --- 1. 상수 정의 ---

/// '방해 앱'으로 간주되는 키워드 목록
/// StateEngine은 이 목록을 기반으로 '이탈 점수'를 계산
const DISTRACTION_KEYWORDS: &[&str] = &[
    "youtube",
    "netflix",
    "facebook",
    "discord",
    "steam.exe",
    "slack",
];

/// 점수 임계값: 이탈 점수가 이 값에 도달하면 '알림'을 트리거
const THRESHOLD_NOTIFICATION: u16 = 10;
/// 점수 임계값: 이탈 점수가 이 값에 도달하면 '강한 개입(오버레이)'을 트리거
const THRESHOLD_OVERLAY: u16 = 20;

/// 점수 계산 규칙: 방해 키워드 발견 시 추가할 점수
const SCORE_DISTRACTION_APP: u16 = 5;
/// 점수 계산 규칙: 3분 (180초) 이상 입력이 없을 시 추가할 점수
const SCORE_INACTIVITY_MILD: u16 = 3;
/// 점수 계산 규칙: 10분 (600초) 이상 입력이 없을 시 추가할 점수
const SCORE_INACTIVITY_SEVERE: u16 = 10;

/// 비활성(Inactivity) 판단 기준 시간 (초 단위)
const INACTIVITY_THRESHOLD_MILD_S: u64 = 180; // 3분
const INACTIVITY_THRESHOLD_SEVERE_S: u64 = 600; // 10분

// --- 2. StateEngine 상태 및 로직 ---

/// State Engine이 반환할 개입(Intervention) 명령의 종류
#[derive(Debug, PartialEq)]
pub enum InterventionTrigger {
    DoNothing,           // 아무것도 하지 않음
    TriggerNotification, // 가벼운 알림 (예: OS 알림)
    TriggerOverlay,      // 강한 개입 (예: 화면 오버레이)
}

/// State Engine의 현재 상태를 관리하는 구조체
/// 이 구조체는 세션이 시작될 때 생성
#[derive(Debug)]
pub struct StateEngine {
    /// 현재 누적된 '이탈 점수'
    deviation_score: u16,
    // 개입 이벤트를 이미 보냈는지 추적하는 플래그
    notification_sent: bool,
    overlay_sent: bool,
}

impl StateEngine {
    /// 새로운 세션을 위한 StateEngine을 생성
    pub fn new() -> Self {
        StateEngine { 
            deviation_score: 0,
            notification_sent: false,
            overlay_sent: false,
         }
    }

    /// 현재 점수를 반환 (UI 표시 등에 사용)
    pub fn get_current_score(&self) -> u16 {
        self.deviation_score
    }

    /// Activity Monitor로부터 받은 최신 데이터를 처리하고,
    /// '이탈 점수'를 갱신한 뒤, 필요한 개입 명령을 반환
    ///
    /// 이 함수는 주기적으로 (예: 매 5초) 또는 이벤트 발생 시 호출
    ///
    /// # Arguments
    /// * `window_info` - 현재 활성 창 정보
    /// * `input_stats` - 현재까지의 누적 입력 통계
    ///
    /// # Returns
    /// * `InterventionTrigger` - 계산 결과에 따른 개입 명령
    pub fn process_activity(
        &mut self,
        window_info: &ActiveWindowInfo,
        input_stats: &InputStats,
    ) -> InterventionTrigger {
        let now_s = current_timestamp_s();
        let score_to_add: u16 = 0;

        // --- 규칙 1: 방해 키워드 검사 ---
        // 창 제목과 앱 이름을 모두 소문자로 변환하여 검사
        let title_lower = window_info.title.to_lowercase();
        let app_name_lower = window_info.app_name.to_lowercase();

        let is_distraction = DISTRACTION_KEYWORDS
            .iter()
            .any(|&keyword| title_lower.contains(keyword) || app_name_lower.contains(keyword));

        // --- 규칙 2: 비활성(Inactivity) 검사 ---
        if is_distraction {
            // --- 2a. 딴짓 중인 경우 ---
            let mut score_to_add: u16 = 0;

            // '딴짓' 자체 점수
            score_to_add += SCORE_DISTRACTION_APP;

            // '딴짓' 중 '비활성' 가중
            let last_input_s = input_stats.last_input_timestamp_ms / 1000;
            let inactivity_duration_s = now_s.saturating_sub(last_input_s);

            if inactivity_duration_s >= INACTIVITY_THRESHOLD_SEVERE_S {
                score_to_add += SCORE_INACTIVITY_SEVERE;
            } else if inactivity_duration_s >= INACTIVITY_THRESHOLD_MILD_S {
                score_to_add += SCORE_INACTIVITY_MILD;
            }

            // 점수를 누적
            self.deviation_score = (self.deviation_score + score_to_add).min(100);

            // 2b. '개입 결정' 로직을 '딴짓' 블록 내부에서만 수행
            if self.deviation_score >= THRESHOLD_OVERLAY {
                if !self.overlay_sent {
                    self.overlay_sent = true; // 플래그 설정
                    return InterventionTrigger::TriggerOverlay;
                }
            } else if self.deviation_score >= THRESHOLD_NOTIFICATION {
                if !self.notification_sent {
                    self.notification_sent = true; // 플래그 설정
                    return InterventionTrigger::TriggerNotification;
                }
            }
            // 이미 알림을 보냈거나, 아직 임계값 미만이면 DoNothing
            return InterventionTrigger::DoNothing; 


        } else {
            // --- 3. 업무 중인 경우 (is_distraction == false) ---

            // 점수를 능동적으로 감소
            self.deviation_score = self.deviation_score.saturating_sub(1);

            // 업무 복귀 시, 모든 플래그를 초기화
            // (다음 '딴짓' 사이클을 위해 재무장)
            if self.notification_sent || self.overlay_sent {
                self.notification_sent = false;
                self.overlay_sent = false;
            }

            // 3b. '업무 중'일 때는 절대 개입하지 않고 DoNothing을 반환
            return InterventionTrigger::DoNothing;
        }
    }
}

// --- 3. 유틸리티 함수 ---

/// 현재 시간을 초 단위 Unix 타임스탬프로 반환
fn current_timestamp_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// --- 4. 유닛 테스트 ---
// 이 모듈이 독립적으로 잘 작동하는지 테스트
#[cfg(test)]
mod tests {
    use super::*;

    // 테스트용 목업(Mock) 데이터 생성
    fn mock_window_info(title: &str, app_name: &str) -> ActiveWindowInfo {
        ActiveWindowInfo {
            timestamp_ms: 0, // 테스트에서는 중요하지 않음
            title: title.to_string(),
            process_path: "".to_string(),
            app_name: app_name.to_string(),
            window_id: "".to_string(),
            process_id: 0,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    fn mock_input_stats(last_input_ago_s: u64) -> InputStats {
        let now_ms = current_timestamp_s() * 1000;
        InputStats {
            total_input_events: 1, // 테스트에서는 중요하지 않음
            last_input_timestamp_ms: now_ms.saturating_sub(last_input_ago_s * 1000),
            start_monitoring_timestamp_ms: 0,
        }
    }

    #[test]
    fn test_distraction_score_increases() {
        let mut engine = StateEngine::new();
        let window_info = mock_window_info("Working on Document", "word.exe");
        let input_stats = mock_input_stats(10); // 10초 전 입력

        // 처음에는 점수가 0
        assert_eq!(engine.get_current_score(), 0);

        // 방해 앱 실행
        let distraction_window = mock_window_info("YouTube - Google Chrome", "chrome.exe");
        let trigger = engine.process_activity(&distraction_window, &input_stats);

        // 점수가 오르고, 알림 트리거 확인 (규칙에 따라 다름)
        assert!(engine.get_current_score() > 0);
        assert_eq!(engine.get_current_score(), SCORE_DISTRACTION_APP);
        // (점수가 THRESHOLD_NOTIFICATION보다 낮다는 가정 하에)
        // assert_eq!(trigger, InterventionTrigger::DoNothing); // 또는 TriggerNotification
    }

    #[test]
    fn test_inactivity_score_only_if_distraction() {
        let mut engine = StateEngine::new();

        // 1. 업무 중 + 비활성 (생각하는 시간)
        let window_info_work = mock_window_info("Working on Document", "word.exe");
        let input_stats_inactive = mock_input_stats(240); // 4분
        engine.process_activity(&window_info_work, &input_stats_inactive);
        // 점수가 오르지 않아야 함 (오히려 0이거나 감소)
        assert_eq!(engine.get_current_score(), 0);

        // 2. 딴짓 중 + 비활성
        let window_info_distraction = mock_window_info("YouTube", "chrome.exe");
        engine.process_activity(&window_info_distraction, &input_stats_inactive);
        // 딴짓 점수 + 비활성 점수
        assert_eq!(
            engine.get_current_score(),
            SCORE_DISTRACTION_APP + SCORE_INACTIVITY_MILD
        );
    }

    #[test]
    fn test_score_decays_when_productive() {
        let mut engine = StateEngine::new();

        // 1. 점수를 먼저 올림 (방해 앱)
        let distraction_window = mock_window_info("YouTube", "chrome.exe");
        let input_stats = mock_input_stats(10);
        engine.process_activity(&distraction_window, &input_stats);
        let initial_score = engine.get_current_score();
        assert_eq!(initial_score, SCORE_DISTRACTION_APP); // 점수: 5

        // 2. 타이머(sleep) 대신, '업무 중' 함수를 1회 호출
        let productive_window = mock_window_info("Productive Task", "code.exe");
        let productive_stats_inactive = mock_input_stats(30); // (생각 중)
        engine.process_activity(&productive_window, &productive_stats_inactive);

        // 점수가 1 감소해야 함
        assert_eq!(engine.get_current_score(), initial_score.saturating_sub(1)); // 점수: 4

        // 3. '업무 중' 함수를 2회 호출
        engine.process_activity(&productive_window, &productive_stats_inactive);

        // 점수가 1 더 감소해야 함
        assert_eq!(engine.get_current_score(), initial_score.saturating_sub(2));
        // 점수: 3
    }

    //  업무 복귀 시 알림이 즉시 멈추는지 테스트
    #[test]
    fn test_intervention_stops_immediately_on_work() {
        let mut engine = StateEngine::new();
        let input_stats = mock_input_stats(10);
        let distraction_window = mock_window_info("YouTube", "chrome.exe");

        // 1. 점수를 20점 (Overlay 임계값)까지 강제로 증가
        let trigger1 = engine.process_activity(&distraction_window, &input_stats); // 5
        let trigger2 = engine.process_activity(&distraction_window, &input_stats); // 10
        let trigger3 = engine.process_activity(&distraction_window, &input_stats); // 15
        let trigger_overlay = engine.process_activity(&distraction_window, &input_stats); // 20

        assert_eq!(engine.get_current_score(), 20);
        assert_eq!(trigger_overlay, InterventionTrigger::TriggerOverlay); // 딴짓 중 -> 오버레이 발생
        assert_eq!(engine.overlay_sent, true); // 플래그 설정 확인

        // 2. 업무 앱으로 전환
        let productive_window = mock_window_info("Productive Task", "code.exe");
        let trigger_work = engine.process_activity(&productive_window, &input_stats);

        // 3. 점수는 19점으로 감소하지만, 반환값은 DoNothing
        assert_eq!(engine.get_current_score(), 19);
        assert_eq!(trigger_work, InterventionTrigger::DoNothing); // [!] 핵심 버그 수정 확인
        assert_eq!(engine.overlay_sent, false); // 플래그 초기화 확인
    }

    // 알림이 단 한 번만 발생하는지 테스트
    #[test]
    fn test_notification_sent_only_once() {
        let mut engine = StateEngine::new();
        let input_stats = mock_input_stats(10);
        let distraction_window = mock_window_info("YouTube", "chrome.exe");

        // 1. 점수를 10점(Notification 임계값)까지 증가
        engine.process_activity(&distraction_window, &input_stats); // 5
        let trigger_notify = engine.process_activity(&distraction_window, &input_stats); // 10
        
        assert_eq!(engine.get_current_score(), 10);
        assert_eq!(trigger_notify, InterventionTrigger::TriggerNotification); // 알림 발생
        assert_eq!(engine.notification_sent, true); // 플래그 설정됨

        // 2. 딴짓을 계속 (점수 15점)
        let trigger_nothing = engine.process_activity(&distraction_window, &input_stats); // 15
        
        assert_eq!(engine.get_current_score(), 15);
        // [!] 점수는 임계값을 넘었지만, 플래그가 설정되어 DoNothing을 반환
        assert_eq!(trigger_nothing, InterventionTrigger::DoNothing); 
    }
}
