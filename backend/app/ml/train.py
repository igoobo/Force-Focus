import os
import json
import numpy as np
import pandas as pd
from datetime import datetime, timezone
from typing import Dict, Any, Optional
from pathlib import Path

from sklearn.svm import OneClassSVM
from sklearn.preprocessing import StandardScaler
from skl2onnx import convert_sklearn
from skl2onnx.common.data_types import FloatTensorType

# [FastAPI 통합] 앱 내부 DB 세션 및 설정을 사용
from app.db.mongo import get_db
from app.core.config import settings

# 모델 저장 경로 (Docker Volume 연동)
BASE_DIR = Path(__file__).resolve().parent.parent.parent
MODEL_STORAGE_PATH = os.path.join(BASE_DIR, "storage", "models")

# ---------------------------------------------------------
# 1. 기존 로직 및 설정 (Global Map & Scoring)
# ---------------------------------------------------------

# [설정] Global Map (Simplified based on Specs)
GLOBAL_MAP = {
    'code': 0.9, 'vs': 0.9, 'intellij': 0.9, 'rust': 0.9, 'py': 0.9,
    'slack': 0.5, 'notion': 0.7, 'github': 0.8, 'stackoverflow': 0.8,
    'arxiv': 0.9,
    'youtube': -0.9, 'netflix': -0.9, 'chzzk': -0.9, 'twitch': -0.9,
    'steam': -0.9, 'game': -0.9, 'lol': -0.9,
    'chrome': 0.1
}

def get_token_score(app_name, title):
    """
    [Simplified] 단일 앱/타이틀에 대한 점수 반환
    - Visual Weighting 제거: Active Window만 고려
    - Simple Tokenization: 공백/특수문자 기준 자르기
    """
    # 1. Combine
    full_text = f"{app_name} {title}".lower()
    
    # 2. Simple Tokenization (non-alphanumeric split)
    tokens = []
    current_token = ""
    for char in full_text:
        if char.isalnum():
            current_token += char
        else:
            if current_token:
                tokens.append(current_token)
                current_token = ""
    if current_token:
        tokens.append(current_token)
    
    # 3. Scoring
    scale_sum = 0.0
    count = 0
    found = False
    
    for t in tokens:
        if not t: continue
        # Exact Match (HashMap lookup)
        if t in GLOBAL_MAP:
            scale_sum += GLOBAL_MAP[t]
            count += 1
            found = True
            
    if not found: return 0.0 # Neutral (Unknown)
    if count == 0: return 0.0
    
    return scale_sum / count

def calculate_context_score_wrapper(row):
    """
    Wrapper for dataframe apply. 
    Only uses 'app_name' and 'window_title' (Active Window).
    """
    return get_token_score(row.get('app_name', ''), row.get('window_title', ''))

# ---------------------------------------------------------
# 2. Main Training Function (Async for FastAPI Integration)
# ---------------------------------------------------------

async def train_user_model(user_id: str) -> Dict[str, Any]:
    """
    [Task 1 & 2] User Isolation + Feedback Filtering
    """
    # [수정] 올바른 DB 객체 호출
    db = get_db()
    
    # 1. Load Data (Async Loop)
    cursor = db.events.find({"user_id": user_id}).sort("timestamp", 1)
    events = await cursor.to_list(length=None)

    if not events or len(events) < 50:
        return {"status": "skipped", "reason": "insufficient_data"}

    # 2. Load Feedback
    feedback_cursor = db.feedback.find({
        "user_id": user_id,
        "feedback_type": "distraction_ignored" 
    })
    ignored_feedbacks = await feedback_cursor.to_list(length=None)
    
    ignored_event_ids = set()
    for fb in ignored_feedbacks:
        # client_event_id 우선, 없으면 event_id(legacy)
        if fb.get("client_event_id"):
            ignored_event_ids.add(fb["client_event_id"])
        elif fb.get("event_id"):
            ignored_event_ids.add(fb["event_id"])

    # 3. Preprocessing
    raw_df = pd.DataFrame(events)
    
    # Filter ignored events
    if 'client_event_id' in raw_df.columns:
        raw_df = raw_df[~raw_df['client_event_id'].isin(ignored_event_ids)]
    
    if len(raw_df) < 10:
        return {"status": "skipped", "reason": "filtered_too_many"}

    # Flatten JSON 'data' field
    if 'data' in raw_df.columns:
        data_df = pd.json_normalize(raw_df['data'])
        df = pd.concat([raw_df.drop(columns=['data']), data_df], axis=1)
    else:
        df = raw_df

    # 4. Feature Engineering (Identical to original)
    df['X_context'] = df.apply(calculate_context_score_wrapper, axis=1)
    
    df['input_count'] = df.get('meaningful_input_events', 0).fillna(0)
    df['delta_input'] = df.groupby('session_id')['input_count'].diff().fillna(0)
    df.loc[df['delta_input'] < 0, 'delta_input'] = 0
    df['X_log_input'] = np.log1p(df['delta_input'])
    
    silence_list = []
    curr = 0
    for val in df['delta_input']:
        if val == 0: curr += 5
        else: curr = 0
        silence_list.append(curr)
    df['X_silence'] = silence_list
    
    df['X_burstiness'] = df['delta_input'].rolling(12, min_periods=1).std().fillna(0)
    
    def check_mouse_active(row):
        evt_time = row.get('timestamp') 
        mouse_ms = row.get('last_mouse_move_timestamp_ms', 0)
        
        if not mouse_ms or pd.isna(evt_time): return 0.0
        
        if evt_time.tzinfo is None:
            evt_time = evt_time.replace(tzinfo=timezone.utc)
            
        evt_ts = evt_time.timestamp()
        mouse_ts = mouse_ms / 1000.0
        
        if 0 <= (evt_ts - mouse_ts) <= 5.0:
            return 1.0
        return 0.0

    df['X_mouse'] = df.apply(check_mouse_active, axis=1)
    
    def sigmoid(x): return 1 / (1 + np.exp(-x))
    df['X_interaction'] = sigmoid(1.0 / (df['delta_input'] + 0.1)) * df['X_context']

    feature_cols = ['X_context', 'X_log_input', 'X_silence', 'X_burstiness', 'X_mouse', 'X_interaction']
    X_df = df[feature_cols].fillna(0.0)
    X = X_df.values

    # 5. Training
    weights = 1 / (1 + np.exp(-(X_df['X_context'] * 5))) * 2.0
    
    scaler = StandardScaler()
    X_scaled = scaler.fit_transform(X)
    
    model = OneClassSVM(kernel='rbf', nu=0.05, gamma='scale')
    model.fit(X_scaled, sample_weight=weights.values)

    # 6. Save (ONNX + JSON)
    version = datetime.now(timezone.utc).strftime("%Y%m%d%H%M%S")
    user_model_dir = os.path.join(MODEL_STORAGE_PATH, user_id, version)
    os.makedirs(user_model_dir, exist_ok=True)

    initial_type = [('float_input', FloatTensorType([None, 6]))]
    onx = convert_sklearn(model, initial_types=initial_type)
    
    onnx_path = os.path.join(user_model_dir, "model.onnx")
    with open(onnx_path, "wb") as f:
        f.write(onx.SerializeToString())

    scaler_params = {
        "mean": scaler.mean_.tolist(),
        "scale": scaler.scale_.tolist(),
        "var": scaler.var_.tolist(),
        "n_samples_seen": int(scaler.n_samples_seen_)
    }
    scaler_path = os.path.join(user_model_dir, "scaler_params.json")
    with open(scaler_path, "w") as f:
        json.dump(scaler_params, f, indent=2)

    # 7. Update User Model Metadata
    await db.user_models.update_one(
        {"user_id": user_id},
        {"$set": {
            "latest_version": version,
            "updated_at": datetime.now(timezone.utc),
            "model_path": onnx_path,
            "scaler_path": scaler_path
        }},
        upsert=True
    )

    return {
        "status": "success",
        "version": version,
        "sample_count": len(X)
    }