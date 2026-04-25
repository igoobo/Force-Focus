# backend/app/models/session.py

from datetime import datetime
from typing import Optional

from pydantic import BaseModel, Field
from bson import ObjectId

from app.models.common import PyObjectId


class SessionInDB(BaseModel):
    """
    MongoDB의 'sessions' 컬렉션에 저장되는 완전한 형태의 데이터 모델입니다.
    """
    id: PyObjectId = Field(default_factory=ObjectId, alias="_id")
    user_id: str
    client_session_id: Optional[str] = None
    task_id: Optional[str] = None

    # ML 모델 도입전 실험적 필드
    profile_id: Optional[str] = None

    start_time: datetime
    end_time: Optional[datetime] = None

    # 세션 종료 후 계산 (초 단위)
    duration: Optional[float] = None

    # active, completed, cancelled
    status: str = "active"

    # 목표 집중 시간 (분 단위)
    goal_duration: Optional[float] = None

    # 시스템 개입 횟수
    interruption_count: int = 0

    model_config = {
        "populate_by_name": True,
        "arbitrary_types_allowed": True,
        "from_attributes": True,
    }