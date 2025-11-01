# 파일 위치: backend/app/models/schedule.py

from pydantic import BaseModel, Field
from datetime import datetime, time
from typing import List, Optional

class ScheduleInDB(BaseModel):
    """
    MongoDB의 'schedules' 컬렉션에 저장되는 완전한 형태의 데이터 모델입니다.
    """
    id: str = Field(..., alias="_id")
    user_id: str
    task_id: Optional[str] = None
    name: str
    start_time: time # HH:MM 형식을 위해 'time' 객체 사용
    end_time: time   # HH:MM 형식을 위해 'time' 객체 사용
    days_of_week: List[int] # 0:월요일 ~ 6:일요일
    created_at: datetime = Field(default_factory=datetime.now)
    is_active: bool = True

    class Config:
        allow_population_by_field_name = True
        orm_mode = True