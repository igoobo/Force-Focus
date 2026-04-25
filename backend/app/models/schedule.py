# backend/app/models/schedule.py

from datetime import datetime, time, date, timezone
from typing import List, Optional

from pydantic import BaseModel, Field
from bson import ObjectId

from app.models.common import PyObjectId


class ScheduleInDB(BaseModel):
    """
    MongoDB의 'schedules' 컬렉션에 저장되는 완전한 형태의 데이터 모델입니다.
    created_at은 timezone-aware UTC 기준으로 저장합니다.
    """
    id: PyObjectId = Field(default_factory=ObjectId, alias="_id")
    user_id: str
    task_id: Optional[str] = None
    name: str
    start_date: Optional[date] = None
    end_date: Optional[date] = None
    description: Optional[str] = None
    start_time: time
    end_time: time
    days_of_week: List[int]
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    is_active: bool = True

    model_config = {
        "populate_by_name": True,
        "from_attributes": True,
        "arbitrary_types_allowed": True,
    }