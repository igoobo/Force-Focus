# backend/app/schemas/schedule.py

from pydantic import BaseModel, Field
from datetime import datetime, time, date
from typing import Optional, Annotated

# --- 타입 별칭 (Pylance 경고 제거 + 검증 규칙 유지) ---
DayOfWeek = Annotated[int, Field(ge=0, le=6)]
DaysOfWeekCreate = Annotated[list[DayOfWeek], Field(min_length=1)]
DaysOfWeekUpdate = list[DayOfWeek] 


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
    # 리스트 내부 요소는 0~6, 최소 1개 이상
    days_of_week: DaysOfWeekCreate
    # [New] 특정 날짜 실행을 위한 필드 (Optional)
    start_date: Optional[date] = None


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
    days_of_week: Optional[DaysOfWeekUpdate] = None
    start_date: Optional[date] = None
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
    days_of_week: list[int]
    start_date: Optional[date] = None # Return as date object (YYYY-MM-DD)
    created_at: datetime
    is_active: bool

    model_config = {
        "from_attributes": True
    }
