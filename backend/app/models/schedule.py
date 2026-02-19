# backend/app/models/schedule.py

from pydantic import BaseModel, Field
from datetime import datetime, time
from typing import List, Optional

from bson import ObjectId
from app.models.common import PyObjectId


class ScheduleInDB(BaseModel):
    """
    MongoDB의 'schedules' 컬렉션에 저장되는 완전한 형태의 데이터 모델입니다.
    """
    id: PyObjectId = Field(default_factory=ObjectId, alias="_id")
    user_id: str
    task_id: Optional[str] = None
    name: str
    start_time: time  # HH:MM 형식을 위해 'time' 객체 사용
    end_time: time    # HH:MM 형식을 위해 'time' 객체 사용
    days_of_week: List[int]  # 0:월요일 ~ 6:일요일
    start_date: Optional[datetime.date] = None # YYYY-MM-DD (Optional)
    created_at: datetime = Field(default_factory=datetime.now)
    is_active: bool = True

    model_config = {
        "populate_by_name": True,
        "from_attributes": True,
        "arbitrary_types_allowed": True,
    }
