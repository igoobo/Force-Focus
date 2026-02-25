use std::collections::{VecDeque, HashMap};
use crate::commands::{WindowInfo, InputStats};

/// 원천 데이터를 ML 모델 입력용 벡터로 변환하는 감각 기관
pub struct FeatureExtractor {
    // Burstiness: 문서 기준 '최근 1분(60초)' 데이터 유지
    input_count_history: VecDeque<f64>, 
    // Delta 계산용
    last_total_events: u64,
    // Context: Global Knowledge (사전 학습된 가중치 맵 시뮬레이션)
    global_weight_map: HashMap<String, f64>,
}

impl FeatureExtractor {
    pub fn new() -> Self {
        // Global Knowledge 초기화 (Synced with backend/train.py)
        // 문서 Phase 2: 보편적 업무/딴짓 구분
        let mut map = HashMap::new();
        // 생산성 도구 (Positive)
        map.insert("code".to_string(), 0.9);      // VSCode
        map.insert("vs".to_string(), 0.9);        // Visual Studio
        map.insert("studio".to_string(), 0.9);    // Android Studio
        map.insert("intellij".to_string(), 0.9);  // IntelliJ
        map.insert("idea".to_string(), 0.9);      // IntelliJ Process Name
        map.insert("rust".to_string(), 0.9);      // Rust 관련 파일
        map.insert("py".to_string(), 0.9);        // Python 파일
        
        map.insert("github".to_string(), 0.8);    // GitHub
        map.insert("stackoverflow".to_string(), 0.8);
        map.insert("arxiv".to_string(), 0.9);     // 논문
        
        map.insert("slack".to_string(), 0.5);     // Communication
        map.insert("notion".to_string(), 0.7);    // Documentation

        // 딴짓 도구 (Negative)
        map.insert("youtube".to_string(), -0.9);
        map.insert("netflix".to_string(), -0.9);
        map.insert("chzzk".to_string(), -0.9);    // Chzzk
        map.insert("twitch".to_string(), -0.9);
        map.insert("steam".to_string(), -0.9);
        map.insert("game".to_string(), -0.9);
        map.insert("lol".to_string(), -0.9);
        
        map.insert("chrome".to_string(), 0.1);    // Neutral (Browser)

        Self {
            input_count_history: VecDeque::with_capacity(60), // 1분
            last_total_events: 0,
            global_weight_map: map,
        }
    }

    pub fn extract(&mut self, window: &WindowInfo, stats: &InputStats, now_ms: u64) -> [f64; 6] {
        // 1. Delta Calculation
        let current_total = stats.meaningful_input_events;
        let delta = if self.last_total_events == 0 && current_total > 0 {
             current_total as f64 
        } else {
             current_total.saturating_sub(self.last_total_events) as f64
        };
        self.last_total_events = current_total;

        // --- Feature 1: Context (Dual-Path Tokenization) ---
        // 문서 명세: 앱 이름과 제목을 분리하여 토큰화하고 가중치 합산
        let context_score = self.calculate_context_score_strict(&window.app_name, &window.title);

        // --- Feature 2: Log Input ---
        let log_input = if delta > 0.0 { (delta + 1.0).ln() } else { 0.0 };

        // --- Feature 3: Silence ---
        let last_input_ms = stats.last_meaningful_input_timestamp_ms;
        let silence_sec = if last_input_ms > 0 {
            now_ms.saturating_sub(last_input_ms) as f64 / 1000.0
        } else {
            0.0
        };

        // --- Feature 4: Burstiness (1 Minute Window) ---
        if self.input_count_history.len() >= 60 {
            self.input_count_history.pop_front();
        }
        self.input_count_history.push_back(delta);
        let burstiness = self.calculate_burstiness();

        // --- Feature 5: Mouse Active Flag ---
        let last_mouse_ms = stats.last_mouse_move_timestamp_ms;
        let mouse_active = if last_mouse_ms > 0 && (now_ms.saturating_sub(last_mouse_ms) <= 5000) {
            1.0
        } else {
            0.0
        };

        // --- Feature 6: Interaction Gate (Sigmoid Logic) ---
        // 문서 수식: Sigmoid(1 / (Input + 0.1)) * Context
        // 입력이 적을수록(침묵) 1.0에 가까워짐 -> 문맥이 중요해짐
        // 입력이 많으면(타이핑) 0.5에 가까워짐 -> 문맥 영향력 감소 (행동 자체가 중요)
        let input_factor = 1.0 / (delta + 0.1); // 분모 0 방지 (Spec: 0.1)
        let sigmoid_val = 1.0 / (1.0 + (-input_factor).exp()); // Basic Sigmoid
        
        // 문서 의도: "입력이 없을 때(Sigmoid High) 문맥이 긍정적이면 Interaction 점수 부여"
        // 범위: 0.0 ~ 1.0 (Interaction은 긍정적 지표이므로 음수 Context는 0처리)
        let interaction_gate = if context_score > 0.0 {
            sigmoid_val * context_score
        } else {
            0.0
        };

        [
            context_score,
            log_input,
            silence_sec,
            burstiness,
            mouse_active,
            interaction_gate
        ]
    }

    /// [Internal] 문서 Phase 2.1 Dual-Path Tokenization 구현
    fn calculate_context_score_strict(&self, app_name: &str, title: &str) -> f64 {
        let mut total_score = 0.0;
        let mut match_count = 0;

        // Tokenizer: 공백 및 특수문자 기준으로 분리
        let full_text = format!("{} {}", app_name, title).to_lowercase();
        let tokens: Vec<&str> = full_text.split(|c: char| !c.is_alphanumeric()).collect();

        for token in tokens {
            if token.is_empty() { continue; }
            
            if let Some(&score) = self.global_weight_map.get(token) {
                total_score += score;
                match_count += 1;
            }
        }

        // 평균값 사용 (단, 매칭된 게 없으면 0.0)
        if match_count > 0 {
            (total_score / match_count as f64).clamp(-1.0, 1.0)
        } else {
            0.0
        }
    }

    /// [Internal] 표준편차 계산 (문서 기준)
    fn calculate_burstiness(&self) -> f64 {
        if self.input_count_history.is_empty() { return 0.0; }
        
        let n = self.input_count_history.len() as f64;
        let sum: f64 = self.input_count_history.iter().sum();
        let mean = sum / n;
        
        if mean == 0.0 { return 0.0; } // 분산 0

        let variance = self.input_count_history.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / n;
        
        variance.sqrt()
    }
}