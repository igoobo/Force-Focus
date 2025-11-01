# 파일 위치: backend/app/models/session.py

from pydantic import BaseModel, Field
from datetime import datetime
from typing import Optional

class SessionInDB(BaseModel):
    """
    MongoDB의 'sessions' 컬렉션에 저장되는 완전한 형태의 데이터 모델입니다.
    데이터 모델링 표의 모든 필드를 포함합니다.
    """
    id: str = Field(..., alias="_id") # MongoDB의 '_id'를 'id'로 매핑
    user_id: str
    task_id: Optional[str] = None # 연결된 할 일이 없을 수도 있습니다.

    # ML 모델 도입전 실험적 필드
    profile_id: Optional[str] = None # 이 세션에 적용된 AI 프로필의 ID 

    start_time: datetime
    end_time: Optional[datetime] = None # 세션 종료 전에는 없을 수 있습니다.
    duration: Optional[float] = None # 세션 종료 후 계산됩니다. (초 단위)
    status: str = "active" # active, completed, cancelled 등
    goal_duration: Optional[float] = None # 목표 집중 시간 (분 단위)
    interruption_count: int = 0 # 시스템 개입 횟수

    

    class Config:
        allow_population_by_field_name = True
        orm_mode = True