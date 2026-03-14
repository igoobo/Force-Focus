use ndarray::Array2;
use ort::session::{Session, builder::GraphOptimizationLevel};
use ort::value::Value; 
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::time::{Instant, Duration};


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
    // ONNX Runtime 세션 (Thread-safe하지 않으므로 &mut 접근 필요)
    session: Option<Session>, // Option으로 감싸서 Unload(None) 상태 허용 -> Windows File Lock 해결
    scaler: ScalerParams,
    
    // Hot-Swap을 위해 경로 기억
    model_path: PathBuf,
    scaler_path: PathBuf,

    // Local Cache: 사용자 피드백(False Positive 신고) 기억 장소
    // Key: App/Title Token, Value: 만료 시간(TTL)
    local_cache: HashMap<String, Instant>,
}

impl InferenceEngine {
    /// 모델과 스케일러를 파일에서 로드하여 엔진 초기화
    pub fn new<P: AsRef<Path>>(model_path: P, scaler_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let (session, scaler) = Self::load_resources(&model_path, &scaler_path)?;

        Ok(Self {
            session: Some(session), // Some으로 감싸기
            scaler,
            model_path: model_path.as_ref().to_path_buf(),
            scaler_path: scaler_path.as_ref().to_path_buf(),
            local_cache: HashMap::new(), // 초기엔 기억 없음
        })
    }

    /// [Internal] 파일 로드 헬퍼 (초기화 및 리로드 공용)
    fn load_resources<P: AsRef<Path>>(model_path: P, scaler_path: P) -> Result<(Session, ScalerParams), Box<dyn std::error::Error>> {
        // A. 스케일러 로드
        let file = File::open(scaler_path)?;
        let reader = BufReader::new(file);
        let scaler: ScalerParams = serde_json::from_reader(reader)?;

        // B. ONNX 모델 로드
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(1)? 
            .commit_from_file(model_path)?;
        
        Ok((session, scaler))
    }

    // ================================================================
    // Windows File Lock 해결을 위한 Lifecycle 메서드
    // ================================================================

    /// Unload: 모델 파일 핸들 해제
    /// 이 함수를 호출하면 Session이 Drop되면서 OS에게 파일 제어권을 반환합니다.
    pub fn unload_model(&mut self) {
        if self.session.is_some() {
            println!("🔻 [InferenceEngine] Unloading model to release file lock...");
            self.session = None; 
        }
    }

    /// Load: 특정 경로의 모델을 다시 로드 (업데이트 후 호출)
    pub fn load_model(&mut self, model_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔺 [InferenceEngine] Loading model from: {:?}", model_path);
        // 스케일러는 기존 경로 재사용 (필요시 인자로 받도록 수정 가능)
        let (session, _) = Self::load_resources(model_path, &self.scaler_path)?;
        self.session = Some(session);
        self.model_path = model_path.to_path_buf();
        Ok(())
    }

    /// Hot-Swap: 실행 중 모델 파일(경로)이 바뀌면 다시 로드
    /// new_model_path: Some(path)가 들어오면 해당 경로로 모델을 교체함. None이면 기존 경로 사용.
    /// Hot-Swap: Unload -> Wait -> Reload 패턴 적용
    pub fn reload(&mut self, new_model_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 [InferenceEngine] Hot-Swap Requested...");
        
        let target_model_path = new_model_path.unwrap_or(self.model_path.clone());

        // 1. 안전한 교체를 위해 Unload 먼저 수행
        self.unload_model();

        // 2. 잠시 대기 (OS가 파일 핸들을 놓을 시간 확보, Windows 이슈 방지)
        // std::thread::sleep은 blocking이지만, 업데이트는 가끔 일어나므로 허용
        std::thread::sleep(std::time::Duration::from_millis(100));

        // 3. 리로드 (파일이 교체되었다고 가정)
        // load_resources를 재사용하여 스케일러와 세션 모두 갱신
        let (new_session, new_scaler) = Self::load_resources(&target_model_path, &self.scaler_path)?;
        
        self.session = Some(new_session);
        self.scaler = new_scaler;
        self.model_path = target_model_path; 
        
        println!("✅ [InferenceEngine] Hot-Swapped Successfully.");
        Ok(())
    }
    // ================================================================
    // 기존 기능 유지 (Infer, Cache Update)
    // ================================================================

    /// Feedback: 사용자가 "나 일하는 중이야"라고 신고하면 캐시에 등록
    /// token: 현재 활성 창의 식별자 (예: "Figma")
    /// ttl_hours: 기억 유지 시간 (보통 24시간)
    pub fn update_local_cache(&mut self, token: String, ttl_hours: u64) {
        let expiration = Instant::now() + Duration::from_secs(ttl_hours * 3600);
        self.local_cache.insert(token.clone(), expiration);
        println!("🧠 Local Cache Updated: '{}' is now trusted until {:?}", token, expiration);
    }
    
    /// 메인 추론 함수
    /// input_vector: FeatureExtractor가 만든 6차원 벡터
    /// active_tokens: 현재 활성 창의 토큰 리스트 (Cache 확인용)
    pub fn infer(&mut self, mut input_vector: [f64; 6], active_tokens: Vec<String>) -> Result<(f64, InferenceResult), Box<dyn std::error::Error>> {
        
        // Session이 None이면 추론 불가 (Early Return)
        let session = match &mut self.session {
            Some(s) => s,
            None => return Err("Model is unloaded. Cannot infer.".into()),
        };

        // 1. Local Cache Check & Override (문서 Phase 5)
        // 사용자가 피드백을 준 토큰(예: "YouTube"로 강의 듣기)이 하나라도 있다면
        // 문맥 점수(0번 인덱스)를 강제로 1.0(만점)으로 수정
        let mut cache_hit = false;
        for token in active_tokens {
            if let Some(expire_time) = self.local_cache.get(&token) {
                if Instant::now() < *expire_time {
                    cache_hit = true;
                    // Note: 하나라도 맞으면 Trusted로 간주
                } else {
                    // 만료된 기억 (Lazy Deletion: 여기서는 삭제 안 하고 넘어가거나, 별도 클린업 필요)
                    // self.local_cache.remove(&token); // loop 중 borrow checker 이슈 가능성 있음
                }
            }
        }
        
        if cache_hit {
             // 캐시 히트! 문맥 점수 강제 상향
             input_vector[0] = 1.0; 
        }

        // 2. Preprocessing (Standard Scaling)
        let mut scaled_input = Array2::<f32>::zeros((1, 6));
        for i in 0..6 {
            let val = (input_vector[i] - self.scaler.mean[i]) / self.scaler.scale[i];
            scaled_input[[0, i]] = val as f32;
        }

        // 3. Inference
        let input_tensor = Value::from_array(scaled_input)?;
        let inputs = ort::inputs![ "float_input" => input_tensor ]; 
        let outputs = session.run(inputs)?;

        let scores = outputs["scores"].try_extract_tensor::<f32>()?;
        let current_score = scores.1[0] as f64;

        // 4. Rule-based Decision
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