# 파일 위치: backend/app/schemas/schedule.py

from pydantic import BaseModel, Field
from datetime import datetime, time
from typing import List, Optional

# --- API 요청(Request) 스키마 ---

class ScheduleCreate(BaseModel):
    """
    [요청] POST /schedules
    새로운 집중 세션 스케줄을 생성할 때 클라이언트가 보내는 데이터 구조입니다.
    """
    task_id: Optional[str] = None
    name: str
    start_time: time
    end_time: time
    days_of_week: List[int] = Field(..., ge=0, le=6) # 0~6 사이의 값만 허용

class ScheduleUpdate(BaseModel):
    """
    [요청] PUT /schedules/{schedule_id}
    기존 스케줄 정보를 업데이트할 때 클라이언트가 보내는 데이터 구조입니다.
    모든 필드는 선택 사항(Optional)입니다.
    """
    task_id: Optional[str] = None
    name: Optional[str] = None
    start_time: Optional[time] = None
    end_time: Optional[time] = None
    days_of_week: Optional[List[int]] = Field(None, ge=0, le=6)
    is_active: Optional[bool] = None

# --- API 응답(Response) 스키마 ---

class ScheduleRead(BaseModel):
    """
    [응답] GET /schedules, POST /schedules 등 조회/생성 성공 시
    데이터베이스에 저장된 스케줄 정보를 클라이언트에게 반환할 때의 데이터 구조입니다.
    """
    id: str
    user_id: str
    task_id: Optional[str] = None
    name: str
    start_time: time
    end_time: time
    days_of_week: List[int]
    created_at: datetime
    is_active: bool

    class Config:
        orm_mode = True