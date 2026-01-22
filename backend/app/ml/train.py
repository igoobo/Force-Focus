import os
import json
import numpy as np
import pandas as pd
from pymongo import MongoClient
from sklearn.svm import OneClassSVM
from sklearn.preprocessing import StandardScaler
from skl2onnx import convert_sklearn
from skl2onnx.common.data_types import FloatTensorType
import joblib
from dotenv import load_dotenv
from datetime import datetime, timezone

# 1. 환경 설정
load_dotenv()
MONGO_URI = os.getenv("MONGO_URI", "mongodb://localhost:27017")
MONGO_DB_NAME = os.getenv("MONGO_DB_NAME", "forcefocus")

# [설정] Global Map (간소화된 버전)
GLOBAL_MAP = {
    'code': 0.9, 'vs': 0.9, 'intellij': 0.9, 'rust': 0.9,
    'slack': 0.5, 'notion': 0.7,
    'youtube': -0.9, 'netflix': -0.9, 'chzzk': -0.9,
    'chrome': 0.1
}

def get_token_score(app_name, title):
    """단일 앱/타이틀에 대한 점수 반환"""
    tokens = [str(app_name).lower().replace('.exe', '')]
    if title:
        tokens.extend(str(title).lower().split())
    
    score = 0.0
    count = 0
    found = False
    for t in tokens:
        for key, val in GLOBAL_MAP.items():
            if key in t:
                score += val
                count += 1
                found = True
    
    if not found: return 0.1 # Unknown
    return score / count

def calculate_visual_context_score(row):
    """
    [Phase 2 Upgraded] Visual Weighting 적용
    활성 창뿐만 아니라, visible_windows의 면적을 고려하여 가중 평균 점수 산출
    """
    # 1. 활성 창 정보
    active_app = row.get('app_name')
    active_title = row.get('window_title')
    
    # 2. Visible Windows 정보 파싱
    visible_windows = row.get('visible_windows')
    
    # 예외 처리: Visible 정보가 없으면 활성 창 점수만 반환
    if not isinstance(visible_windows, list) or not visible_windows:
        return get_token_score(active_app, active_title)
    
    total_area = 0.0
    weighted_score_sum = 0.0
    
    for win in visible_windows:
        # 데이터 구조: { "app_name": ..., "title": ..., "rect": { "top":..., "bottom":... } }
        w_app = win.get('app_name')
        w_title = win.get('title')
        rect = win.get('rect', {})
        
        # 면적 계산 (Width * Height)
        # 좌표가 음수거나 이상할 수 있으므로 절대값/max 처리
        try:
            width = max(0, rect.get('right', 0) - rect.get('left', 0))
            height = max(0, rect.get('bottom', 0) - rect.get('top', 0))
            area = width * height
        except:
            area = 0
            
        if area > 0:
            s = get_token_score(w_app, w_title)
            
            # [가중치 전략] 활성 창(Active)에는 1.5배 가중치를 더 줌 (User Focus 고려)
            if w_app == active_app and w_title == active_title:
                area *= 1.5
                
            weighted_score_sum += s * area
            total_area += area
            
    if total_area == 0:
        return get_token_score(active_app, active_title)
        
    return weighted_score_sum / total_area

def fetch_and_process_data():
    print(f"Connecting to MongoDB: {MONGO_URI}")
    client = MongoClient(MONGO_URI)
    db = client[MONGO_DB_NAME]
    
    # 1. 데이터 가져오기 (visible_windows 포함)
    cursor = db.events.find({}, {
        "timestamp": 1, 
        "activity_vector": 1, 
        "app_name": 1, 
        "window_title": 1, 
        "session_id": 1
    })
    events = list(cursor)
    
    if len(events) < 10:
        return generate_dummy_data()

    print(f"Processing {len(events)} events with Visual Weighting...")
    
    # DataFrame 변환
    raw_df = pd.DataFrame(events)
    # activity_vector 내부 필드(`visible_windows` 등)를 컬럼으로 격상
    activity_df = pd.json_normalize(raw_df['activity_vector'])
    df = pd.concat([raw_df.drop(columns=['activity_vector']), activity_df], axis=1)

    # 2. Feature Engineering
    
    # 2-1. X_context (Visual Weighting 적용!)
    df['X_context'] = df.apply(calculate_visual_context_score, axis=1)
    
    # 2-2. X_log_input
    df['input_count'] = df.get('meaningful_input_events', 0).fillna(0)
    df['delta_input'] = df.groupby('session_id')['input_count'].diff().fillna(0)
    df.loc[df['delta_input'] < 0, 'delta_input'] = 0
    df['X_log_input'] = np.log1p(df['delta_input'])
    
    # 2-3. X_silence
    silence_list = []
    curr = 0
    for val in df['delta_input']:
        if val == 0: curr += 5
        else: curr = 0
        silence_list.append(curr)
    df['X_silence'] = silence_list
    
    # 2-4. X_burstiness
    df['X_burstiness'] = df['delta_input'].rolling(12, min_periods=1).std().fillna(0)
    
    # 2-5. X_mouse (Recency Check 적용!)
    # DB Timestamp(datetime)와 Mouse Timestamp(ms int) 비교
    def check_mouse_active(row):
        evt_time = row.get('timestamp') # datetime
        mouse_ms = row.get('last_mouse_move_timestamp_ms', 0) # int
        
        if not mouse_ms or pd.isna(evt_time): return 0.0
        
        # datetime -> timestamp(s) 변환
        evt_ts = evt_time.replace(tzinfo=timezone.utc).timestamp()
        mouse_ts = mouse_ms / 1000.0
        
        # 5초 이내 움직임이면 활성(1.0)
        if 0 <= (evt_ts - mouse_ts) <= 5.0:
            return 1.0
        return 0.0

    df['X_mouse'] = df.apply(check_mouse_active, axis=1)

    # 2-6. X_interaction
    def sigmoid(x): return 1 / (1 + np.exp(-x))
    df['X_interaction'] = sigmoid(1.0 / (df['delta_input'] + 0.1)) * df['X_context']
    
    return df[['X_context', 'X_log_input', 'X_silence', 'X_burstiness', 'X_mouse', 'X_interaction']].fillna(0.0)

def generate_dummy_data():
    """초기 학습용 더미 데이터"""
    data = {
        'X_context': [0.9]*20 + [-0.9]*10 + [0.1]*10,
        'X_log_input': [2.0]*20 + [0.5]*10 + [0.0]*10,
        'X_silence': [0.0]*20 + [0.0]*10 + [60.0]*10,
        'X_burstiness': [0.5]*40,
        'X_mouse': [1.0]*30 + [0.0]*10,
        'X_interaction': [0.1]*40
    }
    return pd.DataFrame(data)

def train_pipeline():
    # 1. 데이터 준비
    X_df = fetch_and_process_data()
    X = X_df.values
    
    # 2. 가중치 계산 (Opinionated Learning)
    # 문맥 점수가 높을수록 가중치 부여
    weights = 1 / (1 + np.exp(-(X_df['X_context'] * 5))) * 2.0
    
    # 3. Scaling
    print("Scaling features...")
    scaler = StandardScaler()
    X_scaled = scaler.fit_transform(X)
    
    # 4. OC-SVM 학습
    print("Training One-Class SVM...")
    model = OneClassSVM(kernel='rbf', nu=0.05, gamma='scale')
    model.fit(X_scaled, sample_weight=weights.values)
    
    # 5. Export
    os.makedirs("models", exist_ok=True)
    
    # ONNX
    initial_type = [('float_input', FloatTensorType([None, 6]))]
    onx = convert_sklearn(model, initial_types=initial_type)
    with open("models/personal_model.onnx", "wb") as f:
        f.write(onx.SerializeToString())
        
    # Scaler Params (JSON for Rust)
    scaler_params = {
        "mean": scaler.mean_.tolist(),
        "scale": scaler.scale_.tolist(),
        "var": scaler.var_.tolist(), 
        "n_samples_seen": int(scaler.n_samples_seen_)
    }
    with open("models/scaler_params.json", "w") as f:
        json.dump(scaler_params, f, indent=2)
        
    print("Done! Model and Scaler saved in 'backend/models/'")

if __name__ == "__main__":
    train_pipeline()