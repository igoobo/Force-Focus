# 파일 위치: backend/app/models/profile.py

from pydantic import BaseModel, Field
from datetime import datetime
from typing import Dict, List, Any
from enum import Enum

class ModelTypeEnum(str, Enum):
    """프로필이 사용하는 모델의 종류를 정의합니다."""
    TIME_SLICED_RULES = "time_sliced_rules"
    COSINE_SIMILARITY = "cosine_similarity" # 이전 또는 기본 프로필용

class TimeSliceRule(BaseModel):
    """단일 5분 시간 구간에 대한 규칙을 정의합니다."""
    slice_index: int # 0은 0-5분, 1은 5-10분 ...
    rules: Dict[str, Any] # 예: {"typing_freq_min": 0.3, "context_switch_max": 5}

class AIProfileInDB(BaseModel):
    """
    MongoDB의 'AI_Profiles' 컬렉션에 저장되는 완전한 형태의 데이터 모델입니다.
    '다중 프로필' 및 '타임 슬라이싱' 전략을 반영합니다.
    """
    id: str = Field(..., alias="_id")
    user_id: str
    profile_name: str # 예: '코딩 집중', '문서 작업'
    is_default: bool = False # 이 프로필이 사용자의 기본 프로필인지 여부


    model_type: ModelTypeEnum = ModelTypeEnum.TIME_SLICED_RULES

    # ML 모델 도입전 실험적 필드
    time_slices: List[TimeSliceRule] = [] # 5분 간격의 시간대별 규칙 목록


    model_confidence_score: float
    last_updated_at: datetime = Field(default_factory=datetime.utcnow)
    custom_thresholds: Dict[str, float] = {} # 예: {"global_sensitivity": 0.8}

    class Config:
        allow_population_by_field_name = True
        orm_mode = True