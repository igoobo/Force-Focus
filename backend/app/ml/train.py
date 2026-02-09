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
    df['X_context'] = df.apply(calculate_visual_context_score, axis=1)
    
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