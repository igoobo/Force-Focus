import numpy as np
import pandas as pd
from sklearn.svm import OneClassSVM
from sklearn.preprocessing import StandardScaler
from skl2onnx import convert_sklearn
from skl2onnx.common.data_types import FloatTensorType
import json
import os

OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "../../desktop-agent/src-tauri/resources/models")

def generate_raw_timeseries_data():
    """ 
    Rust 백엔드와 완벽히 동일한 환경을 모사하기 위해, 원시 변수(Raw Variables)만 시뮬레이션합니다. 
    X_mouse는 Rust에서 5초 타임 윈도우를 거쳐 0.0 또는 1.0으로 변환되어 들어옴을 전제합니다.
    """
    data = []
    
    # 0.1 ~ 1.0 대역의 범용적 정상 훈련 (연속 분포 학습)
    for ctx in np.arange(0.1, 1.1, 0.1):
        # 시나리오 A: 활성 상태 (키보드 입력 존재, 마우스 5초 내 조작됨=1.0, 침묵 0초)
        for _ in range(300):
            delta = np.random.poisson(10.0)
            data.append({"X_context": ctx, "delta_input": delta, "X_silence": 0.0, "X_mouse": 1.0})
        
        # 시나리오 B: 극단적 침묵 상태 (최대 3시간 방치)
        for silence_val in range(0, 10800, 30):
            # 키보드/마우스 완전 방치 (X_mouse 5초 초과 = 0.0)
            data.append({"X_context": ctx, "delta_input": 0.0, "X_silence": float(silence_val), "X_mouse": 0.0})
            # 마우스만 가끔 까딱 (X_mouse 5초 이내 = 1.0)
            data.append({"X_context": ctx, "delta_input": 0.0, "X_silence": float(silence_val), "X_mouse": 1.0})
            
    return pd.DataFrame(data)

def train_and_export_base_model():
    print("1. Generating raw timeseries data...")
    df = generate_raw_timeseries_data()
    
    print("2. Applying EXACT mathematical feature engineering (Feature Parity with train.py)...")
    # [핵심] train.py 및 Rust AppCore와 100% 동일한 수학적 변환 적용
    df['X_log_input'] = np.log1p(df['delta_input'])
    df['X_burstiness'] = df['delta_input'].rolling(12, min_periods=1).std().fillna(0)
    
    def sigmoid(x): return 1 / (1 + np.exp(-x))
    df['X_interaction'] = sigmoid(1.0 / (df['delta_input'] + 0.1)) * df['X_context']
    
    # 벡터 구성 순서 및 타입 엄격화
    feature_cols = ['X_context', 'X_log_input', 'X_silence', 'X_burstiness', 'X_mouse', 'X_interaction']
    X_df = df[feature_cols].fillna(0.0)
    X_train = X_df.values
    
    print(f"3. Training Baseline OneClassSVM with {len(df)} samples...")
    scaler = StandardScaler()
    X_scaled = scaler.fit_transform(X_train)
    
    # nu=0.05: 결정 경계 외삽 오류 방지를 위한 5% 허용치
    model = OneClassSVM(kernel='rbf', gamma='scale', nu=0.05)
    model.fit(X_scaled)
    
    print("4. Exporting artifacts (ONNX, Scaler, Global Map)...")
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    
    # 4-1. ONNX 모델
    initial_type = [('float_input', FloatTensorType([None, 6]))]
    onnx_model = convert_sklearn(model, initial_types=initial_type)
    onnx_path = os.path.join(OUTPUT_DIR, "personal_model.onnx")
    with open(onnx_path, "wb") as f: 
        f.write(onnx_model.SerializeToString())
        
    # 4-2. Scaler
    scaler_params = {"mean": scaler.mean_.tolist(), "scale": scaler.scale_.tolist()}
    scaler_path = os.path.join(OUTPUT_DIR, "scaler_params.json")
    with open(scaler_path, "w") as f: 
        json.dump(scaler_params, f)
        
    # 4-3. Global Map (Rule Export)
    # 4-3. Global Map (Rule Export)
    global_map = {
        'code': 0.9, 'vs': 0.9, 'intellij': 0.9, 'rust': 0.9, 'py': 0.9,
        'slack': 0.5, 'notion': 0.7, 'github': 0.8, 'stackoverflow': 0.8,
        'arxiv': 0.9,
        'youtube': -0.9, 'netflix': -0.9, 'chzzk': -0.9, 'twitch': -0.9,
        'steam': -0.9, 'game': -0.9, 'lol': -0.9,
        'chrome': 0.1
    }
    map_path = os.path.join(OUTPUT_DIR, "global_map.json")
    with open(map_path, "w") as f:
        json.dump(global_map, f)
        
    print(f"✅ All artifacts perfectly synchronized and saved to {OUTPUT_DIR}")

if __name__ == "__main__":
    train_and_export_base_model()