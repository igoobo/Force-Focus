use ndarray::{Array2, Array1};
use ort::session::{Session, builder::GraphOptimizationLevel};
use ort::value::Value; 
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Mutex;

// 1. JSON 스케일러 파라미터 구조체
// backend/train.py에서 저장한 scaler_params.json과 매핑됨
#[derive(Debug, Deserialize)]
pub struct ScalerParams {
    pub mean: Vec<f64>,
    pub scale: Vec<f64>,
    // var, n_samples_seen은 추론에 불필요하므로 무시
}

// 2. TTA(Test-Time Adaptation) 상태 관리
// 사용자의 컨디션 변화를 추적하는 동적 메모리
pub struct TTAState {
    pub moving_avg_score: f64, // 점수 이동 평균 (Momentum)
    pub alpha: f64,            // 모멘텀 계수 (과거의 영향을 얼마나 유지할지)
}

// 3. 메인 추론 엔진
pub struct InferenceEngine {
    session: Session,            // ONNX 런타임 세션
    scaler: ScalerParams,        // 정규화 파라미터
    tta_state: Mutex<TTAState>,  // 내부 가변성(Interior Mutability)을 위한 Mutex
}

impl InferenceEngine {
    /// 모델과 스케일러를 파일에서 로드하여 엔진 초기화
    pub fn new<P: AsRef<Path>>(model_path: P, scaler_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        // A. 스케일러 로드
        let file = File::open(scaler_path)?;
        let reader = BufReader::new(file);
        let scaler: ScalerParams = serde_json::from_reader(reader)?;

        // B. ONNX 모델 로드 (ort 2.0 API)
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(1)? // 백그라운드 앱이므로 스레드 최소화
            .commit_from_file(model_path)?;

        Ok(Self {
            session,
            scaler,
            // TTA 초기 상태 설정
            tta_state: Mutex::new(TTAState {
                moving_avg_score: 0.0, // 0.0에서 시작 (중립)
                alpha: 0.95,           // 0.95: 과거 데이터를 95% 반영 (서서히 변함)
            }),
        })
    }

    /// 6차원 벡터를 받아 (점수, 이탈여부) 반환
    pub fn infer(&mut self, input_vector: [f64; 6]) -> Result<(f64, bool, f64), Box<dyn std::error::Error>> {
        // 1. Preprocessing (Standard Scaling)
        // 수식: z = (x - mean) / scale
        let mut scaled_input = Array2::<f32>::zeros((1, 6));
        for i in 0..6 {
            let val = (input_vector[i] - self.scaler.mean[i]) / self.scaler.scale[i];
            scaled_input[[0, i]] = val as f32;
        }

        // 2. ONNX Inference
        // "float_input"은 train.py에서 지정한 입력 노드 이름
        let input_tensor = Value::from_array(scaled_input)?;
        let inputs = ort::inputs![ "float_input" => input_tensor ];

        let outputs = self.session.run(inputs)?;

        // Output extraction
        // sklearn OneClassSVM outputs:
        // scores.0은 모양, scores.1은 데이터입니다.
        let scores = outputs["scores"].try_extract_tensor::<f32>()?;
        // 튜플의 두 번째 요소(데이터)에서 첫 번째 값(0번 인덱스)을 가져옵니다.
        let current_score = scores.1[0] as f64;

        // 3. TTA Logic (Adaptive Thresholding)
        let mut tta = self.tta_state.lock().unwrap();
        
        // A. Momentum Update (지수 이동 평균)
        // mu_t = alpha * mu_{t-1} + (1 - alpha) * score_t
        tta.moving_avg_score = tta.alpha * tta.moving_avg_score + (1.0 - tta.alpha) * current_score;
        
        // B. Adaptive Threshold Calculation
        // 평소 점수가 높으면 기준도 높아지고(엄격), 낮으면 낮아짐(관대)
        // 단, 너무 극단적으로 변하지 않게 Clamp(-0.2 ~ 0.2) 적용
        let adaptive_threshold = (tta.moving_avg_score * 0.5).clamp(-0.2, 0.2);

        // 4. Final Decision
        // 점수가 임계값보다 낮으면 '이탈(Anomaly)'
        let is_anomaly = current_score < adaptive_threshold;

        Ok((current_score, is_anomaly, adaptive_threshold))
    }
}