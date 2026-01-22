use ndarray::{Array2, Array1};
use ort::session::{Session, builder::GraphOptimizationLevel};
use ort::value::Value; 
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;


// 1. JSON 스케일러 파라미터 구조체
// backend/train.py에서 저장한 scaler_params.json과 매핑됨
#[derive(Debug, Deserialize)]
pub struct ScalerParams {
    pub mean: Vec<f64>,
    pub scale: Vec<f64>,
    // var, n_samples_seen은 추론에 불필요하므로 무시
}

// 2. 문서 명시된 판단 결과 열거형
#[derive(Debug, Clone, PartialEq)]
pub enum InferenceResult {
    Inlier,       // 정상 (Score > 0.0)
    WeakOutlier,  // 애매한 이탈 (-0.5 < Score <= 0.0)
    StrongOutlier // 확정적 이탈 (Score <= -0.5)
}

// 3. 메인 추론 엔진
pub struct InferenceEngine {
    session: Session,            
    scaler: ScalerParams,        
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
        })
    }

    /// 6차원 벡터를 받아 문서에 정의된 3단계 상태를 반환
    /// 반환값: (Score, InferenceResult)
    pub fn infer(&mut self, input_vector: [f64; 6]) -> Result<(f64, InferenceResult), Box<dyn std::error::Error>> {
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

        // 3. Rule-based Decision
        let judgment = if current_score > 0.0 {
            InferenceResult::Inlier
        } else if current_score > -0.5 {
            InferenceResult::WeakOutlier
        } else {
            InferenceResult::StrongOutlier
        };

        Ok((current_score, judgment))
    }
}