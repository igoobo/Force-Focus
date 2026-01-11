// 파일 위치: src-tauri/src/state_engine.rs

use crate::commands::{ActiveWindowInfo, InputStats};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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

// '업무'로 점수를 감소시키는 데 필요한 시간 (초)
const DECAY_INTERVAL_S: u64 = 10; // 10초마다 1점 감소

// 다시 개입하기 위한 대기 시간
const SNOOZE_DURATION_S: u64 = 10;

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

    //  마지막 점수 감소 시점
    last_decay_timestamp_s: u64,

    // '스누즈' 타이머 (마지막 개입 시간)
    last_intervention_level: u16, // 0=None, 1=Notification, 2=Overlay
    last_intervention_timestamp_s: u64,
}

impl StateEngine {
    /// 새로운 세션을 위한 StateEngine을 생성
    pub fn new() -> Self {
        StateEngine {
            deviation_score: 0,

            // 타이머 초기화
            last_decay_timestamp_s: current_timestamp_s(),

            // 스누즈 타이머 초기화 (0으로 설정하여 앱 시작 시 즉시 개입 가능하도록)
            last_intervention_level: 0,
            last_intervention_timestamp_s: 0,
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
            let last_input_s = input_stats.last_meaningful_input_timestamp_ms / 1000;
            let inactivity_duration_s = now_s.saturating_sub(last_input_s);

            if inactivity_duration_s >= INACTIVITY_THRESHOLD_SEVERE_S {
                score_to_add += SCORE_INACTIVITY_SEVERE;
            } else if inactivity_duration_s >= INACTIVITY_THRESHOLD_MILD_S {
                score_to_add += SCORE_INACTIVITY_MILD;
            }

            // 점수를 누적
            self.deviation_score = (self.deviation_score + score_to_add).min(100);

            // 딴짓 중일 때는 '점수 감소 타이머'를 현재 시간으로 리셋
            self.last_decay_timestamp_s = now_s;

            // 2b. '개입 결정' 로직:  '스누즈 타이머' 검사
            // 마지막 개입 이후 '스누즈 시간'(10초)이 지났는지 확인
            let time_since_last_intervention =
                now_s.saturating_sub(self.last_intervention_timestamp_s);
            let is_snooze_over = time_since_last_intervention >= SNOOZE_DURATION_S;

            if self.deviation_score >= THRESHOLD_OVERLAY {
                // [수준 2: Overlay]
                // '이전 개입'이 'Overlay'(수준 2) 미만이었거나, 스누즈가 끝났다면
                if self.last_intervention_level < 2 || is_snooze_over {
                    self.last_intervention_level = 2;
                    self.last_intervention_timestamp_s = now_s; // 스누즈 타이머 리셋
                    return InterventionTrigger::TriggerOverlay;
                }
            } else if self.deviation_score >= THRESHOLD_NOTIFICATION {
                // [수준 1: Notification]
                // '이전 개입'이 'Notification'(수준 1) 미만이었거나, 스누즈가 끝났다면
                if self.last_intervention_level < 1 || is_snooze_over {
                    self.last_intervention_level = 1;
                    self.last_intervention_timestamp_s = now_s; // 스누즈 타이머 리셋
                    return InterventionTrigger::TriggerNotification;
                }
            }

            // 아직 임계값 미만이면 DoNothing
            return InterventionTrigger::DoNothing;
        } else {
            // --- 3. 업무 중인 경우 (is_distraction == false) ---

            // '시간 기반'으로 점수를 감소
            let time_since_last_decay = now_s.saturating_sub(self.last_decay_timestamp_s);

            if time_since_last_decay >= DECAY_INTERVAL_S {
                self.deviation_score = self.deviation_score.saturating_sub(1);
                // 점수 감소 타이머를 리셋
                self.last_decay_timestamp_s = now_s;
            }

            // '쿨다운' 로직: 점수가 낮아지면 '개입 수준'을 리셋
            if self.deviation_score < THRESHOLD_NOTIFICATION && self.last_intervention_level > 0 {
                self.last_intervention_level = 0;
                self.last_intervention_timestamp_s = 0; // 스누즈 타이머 리셋
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
    use crate::commands::{ActiveWindowInfo, InputStats};
    use std::time::Duration; // [추가] 10초 쿨다운을 시뮬레이션

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

    // 'InputStats' 맞게 목업 데이터 생성
    fn mock_input_stats(last_input_ago_s: u64) -> InputStats {
        let now_ms = current_timestamp_s() * 1000;
        InputStats {
            meaningful_input_events: 1,
            last_meaningful_input_timestamp_ms: now_ms.saturating_sub(last_input_ago_s * 1000),
            last_mouse_move_timestamp_ms: now_ms,
            start_monitoring_timestamp_ms: 0,
            visible_windows: Vec::new(),
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

        // 2. 'DECAY_INTERVAL_S' (10초) 이상 시간이 흐르도록 시뮬레이션
        std::thread::sleep(Duration::from_secs(DECAY_INTERVAL_S + 1));

        // 3. '업무 중' 함수를 1회 호출
        let productive_window = mock_window_info("Productive Task", "code.exe");
        let productive_stats_inactive = mock_input_stats(30); // (생각 중)
        engine.process_activity(&productive_window, &productive_stats_inactive);

        //  10초가 지났으므로 점수가 1 감소해야 함
        assert_eq!(engine.get_current_score(), initial_score.saturating_sub(1)); // 점수: 4

        // 4. 10초가 흐르기 *전*에 '업무 중' 함수를 다시 호출
        engine.process_activity(&productive_window, &productive_stats_inactive);

        //  쿨다운이 갱신되지 않았으므로 점수가 4점으로 유지되어야 함
        assert_eq!(engine.get_current_score(), initial_score.saturating_sub(1));
    }

    //  업무 복귀 시 알림이 즉시 멈추는지 테스트
    #[test]
    fn test_intervention_stops_immediately_on_work_and_escalates() {
        let mut engine = StateEngine::new();
        let input_stats = mock_input_stats(10);
        let distraction_window = mock_window_info("YouTube", "chrome.exe");

        // 1. 점수를 20점 (Overlay)까지 강제로 증가
        engine.process_activity(&distraction_window, &input_stats); // 5 (Lvl 0)
        let trigger_notify = engine.process_activity(&distraction_window, &input_stats); // 10 (Lvl 1)
        let trigger_nothing = engine.process_activity(&distraction_window, &input_stats); // 15 (Lvl 1, Snooze)
        let trigger_overlay = engine.process_activity(&distraction_window, &input_stats); // 20 (Lvl 2)

        // Notify(Lvl 1)는 Overlay(Lvl 2)를 막지 못해야 함
        assert_eq!(trigger_notify, InterventionTrigger::TriggerNotification);
        assert_eq!(trigger_nothing, InterventionTrigger::DoNothing);
        assert_eq!(trigger_overlay, InterventionTrigger::TriggerOverlay);
        assert_eq!(engine.get_current_score(), 20);
        assert_eq!(engine.last_intervention_level, 2);

        // 2. 업무 앱으로 전환
        let productive_window = mock_window_info("Productive Task", "code.exe");
        let trigger_work = engine.process_activity(&productive_window, &input_stats);

        // 3. 점수는 20으로 *유지* (쿨다운 전)
        assert_eq!(engine.get_current_score(), 20);
        // 4. 하지만 반환값은 DoNothing (개입 즉시 멈춤)
        assert_eq!(trigger_work, InterventionTrigger::DoNothing);

        // 5.  쿨다운 로직이 '개입 수준'을 리셋하는지 검증 (루프 사용)
        // (점수를 20점에서 9점으로, 총 11점 감소시켜야 함)
        for i in 0..11 {
            // [!] 10초(DECAY_INTERVAL_S) 이상 시간이 흐르도록 시뮬레이션
            std::thread::sleep(Duration::from_secs(DECAY_INTERVAL_S + 1));

            // [!] 쿨다운 로직을 1회 실행
            engine.process_activity(&productive_window, &input_stats);

            //  점수가 1점 감소했는지 확인 (20->19, 19->18 ...)
            assert_eq!(engine.get_current_score(), 20 - (i + 1));
        }

        // 11번의 루프 후 점수가 9점이 되었는지 확인
        assert_eq!(engine.get_current_score(), 9);
        // 점수가 10점 미만이 되었으므로, 개입 수준이 0으로 리셋되었는지 확인
        assert_eq!(engine.last_intervention_level, 0);
    }

    //  알림이 '한 번만'이 아니라 '10초 스누즈' 후에
    // 다시 발생하는지 검증
    #[test]
    fn test_notification_snoozes_and_resets() {
        let mut engine = StateEngine::new();
        let input_stats = mock_input_stats(10);
        let distraction_window = mock_window_info("YouTube", "chrome.exe");

        // 1. 10점 도달 -> 알림 1회 발생
        engine.process_activity(&distraction_window, &input_stats); // 5
        let trigger_notify = engine.process_activity(&distraction_window, &input_stats); // 10

        assert_eq!(trigger_notify, InterventionTrigger::TriggerNotification); // 알림 발생
        let first_intervention_time = engine.last_intervention_timestamp_s;
        assert_eq!(engine.last_intervention_level, 1);

        // 2. 딴짓을 계속 (점수 15점)
        let trigger_nothing = engine.process_activity(&distraction_window, &input_stats); // 15

        assert_eq!(engine.get_current_score(), 15);
        // [!] 스누즈(10초)가 활성 중이고, 레벨(1)이 오르지 않았으므로 DoNothing
        assert_eq!(trigger_nothing, InterventionTrigger::DoNothing);
        assert_eq!(
            engine.last_intervention_timestamp_s,
            first_intervention_time
        );

        // 3. 'SNOOZE_DURATION_S' (10초) 이상 시간이 흐르도록 시뮬레이션
        std::thread::sleep(Duration::from_secs(SNOOZE_DURATION_S + 1));

        // 4. 딴짓을 계속 (점수 20점 -> Overlay로 에스컬레이션)
        let trigger_notify_again = engine.process_activity(&distraction_window, &input_stats);

        assert_eq!(engine.get_current_score(), 20);
        // [!] 10초가 지났으므로, 스누즈가 풀리고 '다시' 개입해야 함
        assert_eq!(trigger_notify_again, InterventionTrigger::TriggerOverlay);
        assert!(engine.last_intervention_timestamp_s > first_intervention_time);
    }
}
