use serde::{Deserialize, Serialize};
use crate::inference::InferenceResult;

// --- 1. 설정값 (시간 단위: 초) ---
// 문서 Phase 4-2.A: 상태 정의 및 임계값
const THRESHOLD_NOTIFY_SEC: f64 = 30.0;  // 30초: DRIFT 진입 (알림)
const THRESHOLD_BLOCK_SEC: f64 = 60.0;   // 60초: DISTRACTED 진입 (차단)
const SNOOZE_SEC: f64 = 10.0;            // 개입 후 10초간 대기 (피로도 관리)

// --- 2. 개입 트리거 (3단계) ---
#[derive(Debug, Clone, PartialEq)]
pub enum InterventionTrigger {
    DoNothing,          // 평화
    TriggerNotification, // 주의 환기
    TriggerOverlay,      // 강제 차단
}

// --- 3. FSM 상태 정의 ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FSMState {
    IDLE,       // (초기 상태)
    FOCUS,      // 몰입 상태 (게이지 < 30)
    DRIFT,      // 유예 상태 (30 <= 게이지 < 60)
    DISTRACTED, // 이탈 상태 (게이지 >= 60)
}

// --- 4. 상태 엔진 구조체 ---
pub struct StateEngine {
    current_state: FSMState,
    
    // [Core] 누적 이탈 시간 (초 단위 게이지)
    // 0.0 에서 시작하여 조건에 따라 차오르거나 줄어듦
    drift_gauge: f64,
    
    // Delta Time 계산용
    last_tick_ts: u64,
    
    // 스누즈(재알림 방지) 타이머 (마지막 개입 시각)
    last_intervention_ts: f64, 
}

impl StateEngine {
    pub fn new() -> Self {
        Self {
            current_state: FSMState::IDLE,
            drift_gauge: 0.0,
            last_tick_ts: 0,
            last_intervention_ts: 0.0,
        }
    }

    /// [Process] 매 틱(Tick)마다 호출되어 상태를 갱신하고 행동을 결정
    /// - inference: ML 모델의 판단
    /// - now_ts: 현재 시스템 시간
    /// - is_mouse_active: Safety 2 적용을 위한 마우스 상태
    /// - has_recent_input: Safety 2 적용을 위한 입력 여부 (키보드 등)
    pub fn process(
        &mut self, 
        inference: &InferenceResult, 
        now_ts: u64,
        is_mouse_active: bool,
        has_recent_input: bool
    ) -> InterventionTrigger {
        
        // 1. Delta Time (dt) 계산
        let dt = if self.last_tick_ts == 0 { 
            0.0 
        } else { 
            (now_ts.saturating_sub(self.last_tick_ts)) as f64 
        };
        self.last_tick_ts = now_ts;
        let now_sec = now_ts as f64;

        // 2. 게이지 업데이트 (Time Integration)
        let multiplier = self.calculate_multiplier(inference, is_mouse_active, has_recent_input);
        
        // 게이지 누적/감소 (최소 0.0, 최대 차단 임계값 + 여유분까지 허용)
        self.drift_gauge = (self.drift_gauge + (dt * multiplier)).max(0.0);
        
        // (Optional) 디버깅용: 게이지 상태 출력
        // println!("Gauge: {:.2}s (x{:.1}) | State: {:?}", self.drift_gauge, multiplier, self.current_state);

        // 3. 상태 전이 (Threshold Check)
        self.update_state();

        // 4. 행동 결정 (Snooze Logic)
        self.decide_intervention(now_sec)
    }

    /// [Internal] 상황별 시간 가중치 계산 (문서 Phase 4-2.B & 4-3)
    fn calculate_multiplier(
        &self, 
        inference: &InferenceResult, 
        is_mouse_active: bool, 
        has_recent_input: bool
    ) -> f64 {
        match inference {
            InferenceResult::StrongOutlier => {
                // 문서: StrongOutlier는 급박한 이탈 -> 1.0배속 (또는 더 빠르게 설정 가능)
                1.0 
            },
            InferenceResult::WeakOutlier => {
                // 기본 WeakOutlier는 0.5배속 (시간 지연)
                let mut speed = 0.5;
                
                // [Safety 2: Active Thinking Protection]
                // 문서: "Input은 0이지만 Mouse는 움직임" -> 속도를 절반으로 줄임
                if !has_recent_input && is_mouse_active {
                    speed *= 0.5; // 즉, 0.25배속이 됨
                }
                speed
            },
            InferenceResult::Inlier => {
                // [Fast Recovery]
                // 문서: 업무 복귀 시 빠르게 게이지 감소 (-2.0배속)
                -2.0
            }
        }
    }

    /// [Internal] 게이지 수위에 따른 상태 변경
    fn update_state(&mut self) {
        let next_state = if self.drift_gauge >= THRESHOLD_BLOCK_SEC {
            FSMState::DISTRACTED
        } else if self.drift_gauge >= THRESHOLD_NOTIFY_SEC {
            FSMState::DRIFT
        } else {
            FSMState::FOCUS
        };

        if next_state != self.current_state {
            // println!("🔄 State Transition: {:?} -> {:?}", self.current_state, next_state);
            self.current_state = next_state;
        }
    }

    /// [Internal] 개입 여부 결정 (Snooze 적용)
    fn decide_intervention(&mut self, now_sec: f64) -> InterventionTrigger {
        // 스누즈 체크: 마지막 개입 후 10초가 지났는가?
        if (now_sec - self.last_intervention_ts) < SNOOZE_SEC {
            return InterventionTrigger::DoNothing;
        }

        match self.current_state {
            FSMState::DISTRACTED => {
                // 차단 단계
                self.last_intervention_ts = now_sec;
                InterventionTrigger::TriggerOverlay
            },
            FSMState::DRIFT => {
                // 알림 단계
                self.last_intervention_ts = now_sec;
                InterventionTrigger::TriggerNotification
            },
            _ => InterventionTrigger::DoNothing,
        }
    }
    
    // UI 표시용 Getter
    pub fn get_gauge_ratio(&self) -> f64 {
        (self.drift_gauge / THRESHOLD_BLOCK_SEC).min(1.0)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::InferenceResult;

    // 테스트 헬퍼: 엔진을 만들고 특정 상태로 틱(Tick)을 진행시킴
    fn simulate_ticks(
        engine: &mut StateEngine, 
        seconds: u64, 
        inference: InferenceResult,
        mouse: bool,
        input: bool
    ) -> InterventionTrigger {
        let start_time = engine.last_tick_ts.max(1000); // 0 방지
        let mut last_trigger = InterventionTrigger::DoNothing;

        for i in 1..=seconds {
            // 1초씩 시간 증가 시뮬레이션
            last_trigger = engine.process(
                &inference, 
                start_time + i, 
                mouse, 
                input
            );
        }
        last_trigger
    }

    #[test]
    fn test_strong_outlier_accumulation() {
        let mut engine = StateEngine::new();
        engine.last_tick_ts = 1000; // 초기화

        // 1. Strong Outlier 30초 지속 -> 게이지 30 (1.0배속)
        // -> DRIFT 진입 -> Notification 발생
        let trigger = simulate_ticks(&mut engine, 30, InferenceResult::StrongOutlier, false, false);
        
        assert_eq!(engine.current_state, FSMState::DRIFT);
        assert_eq!(trigger, InterventionTrigger::TriggerNotification);
        assert!((engine.drift_gauge - 30.0).abs() < 0.1); // 부동소수점 오차 허용 비교
    }

    #[test]
    fn test_weak_outlier_time_dilation() {
        let mut engine = StateEngine::new();
        engine.last_tick_ts = 1000;

        // 1. Weak Outlier 30초 지속 -> 게이지 15 (0.5배속)
        // -> 아직 FOCUS 상태여야 함 (임계값 30 미만)
        let trigger = simulate_ticks(&mut engine, 30, InferenceResult::WeakOutlier, false, false);
        
        assert_eq!(engine.current_state, FSMState::FOCUS);
        assert_eq!(trigger, InterventionTrigger::DoNothing);
        assert!((engine.drift_gauge - 15.0).abs() < 0.1);
    }

    #[test]
    fn test_safety_net_active_thinking() {
        let mut engine = StateEngine::new();
        engine.last_tick_ts = 1000;

        // 1. Weak Outlier지만 마우스가 움직임 (Safety 2) -> 0.25배속
        // 40초 흐름 -> 게이지 10 증가 (40 * 0.25)
        simulate_ticks(&mut engine, 40, InferenceResult::WeakOutlier, true, false);
        
        assert!((engine.drift_gauge - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_fast_recovery() {
        let mut engine = StateEngine::new();
        engine.last_tick_ts = 1000;
        engine.drift_gauge = 30.0; // 이미 DRIFT 상태라고 가정
        engine.current_state = FSMState::DRIFT;

        // 1. 업무 복귀(Inlier) 10초 -> 게이지 20 감소 (2.0배속) -> 10 남음
        simulate_ticks(&mut engine, 10, InferenceResult::Inlier, false, true);
        
        assert!((engine.drift_gauge - 10.0).abs() < 0.1);
        
        // 2. 추가 5초 -> 게이지 0 (음수 방지 확인)
        simulate_ticks(&mut engine, 5, InferenceResult::Inlier, false, true);
        assert_eq!(engine.drift_gauge, 0.0);
    }

    #[test]
    fn test_transition_and_snooze() {
        let mut engine = StateEngine::new();
        engine.last_tick_ts = 1000;

        // 1. 30초 딴짓 -> Notification 발동 (시각: 1030)
        let t1 = simulate_ticks(&mut engine, 30, InferenceResult::StrongOutlier, false, false);
        assert_eq!(t1, InterventionTrigger::TriggerNotification);

        // 2. 바로 다음 1초 딴짓 -> Snooze 때문에 DoNothing (시각: 1031, 경과: 1초)
        let t2 = simulate_ticks(&mut engine, 1, InferenceResult::StrongOutlier, false, false);
        assert_eq!(t2, InterventionTrigger::DoNothing);

        // 3. 8초 더 딴짓 -> 총 9초 경과 (시각: 1039, 경과: 9초)
        simulate_ticks(&mut engine, 8, InferenceResult::StrongOutlier, false, false);

        // 4. 1초 더 딴짓 -> 총 10초 경과 (시각: 1040, 경과: 10초) -> Snooze 만료, Notification 발동
        let t3 = simulate_ticks(&mut engine, 1, InferenceResult::StrongOutlier, false, false);
        assert_eq!(t3, InterventionTrigger::TriggerNotification);
        
        // 5. 20초 더 딴짓 -> 게이지 60 도달 -> Overlay 발동
        // (현재 게이지 약 40초 + 20초 = 60초)
        let t4 = simulate_ticks(&mut engine, 20, InferenceResult::StrongOutlier, false, false);
        assert_eq!(t4, InterventionTrigger::TriggerOverlay);
    }
}