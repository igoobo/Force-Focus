# 파일 위치: backend/app/schemas/profile.py

from pydantic import BaseModel
from datetime import datetime
from typing import Dict, List, Any, Optional
from app.models.profile import ModelTypeEnum, TimeSliceRule # models에서 정의한 Enum, Class 재사용

# --- API 요청(Request) 스키마 ---
class AIProfileCreate(BaseModel):
    """[요청] POST /profiles - 새로운 프로필 생성 시"""
    profile_name: str

class AIProfileUpdate(BaseModel):
    """[요청] PUT /profiles/{profile_id} - 프로필 수정 시"""
    profile_name: Optional[str] = None
    custom_thresholds: Optional[Dict[str, float]] = None

# --- API 응답(Response) 스키마 ---
class AIProfileRead(BaseModel):
    """[응답] GET /profiles, GET /profiles/{profile_id} 등 조회 시"""
    id: str
    user_id: str
    profile_name: str
    is_default: bool
    model_type: ModelTypeEnum
    time_slices: List[TimeSliceRule]
    model_confidence_score: float
    last_updated_at: datetime
    custom_thresholds: Dict[str, float]

    class Config:
        orm_mode = True