# backend/app/schemas/schedule.py

from datetime import datetime, time, date
from typing import Optional, Annotated

from pydantic import BaseModel, Field, ConfigDict, field_validator, model_validator

DayOfWeek = Annotated[int, Field(ge=0, le=6)]
DaysOfWeekCreate = Annotated[list[DayOfWeek], Field(min_length=1)]
DaysOfWeekUpdate = Annotated[list[DayOfWeek], Field(min_length=1)]


def _strip_and_reject_blank(v: str, field_name: str) -> str:
    """
    문자열 양쪽 공백 제거 후,
    빈 문자열이면 ValidationError 유발을 위해 ValueError 발생.
    """
    if v is None:
        return v
    if not isinstance(v, str):
        return v
    stripped = v.strip()
    if stripped == "":
        raise ValueError(f"{field_name} must not be blank")
    return stripped


def _strip_to_none(v):
    """
    Optional[str] 입력에서:
    - None은 그대로
    - "   " -> None
    - 그 외는 strip된 문자열
    """
    if v is None:
        return None
    if not isinstance(v, str):
        return v
    s = v.strip()
    return s or None


class ScheduleCreate(BaseModel):
    """
    [요청] POST /schedules

    클라이언트 입력 형식:
    - start_time, end_time: HH:MM[:SS]
    - start_date, end_date: YYYY-MM-DD
    """
    model_config = ConfigDict(str_strip_whitespace=True)

    task_id: Optional[str] = None
    name: str
    start_date: Optional[date] = None
    end_date: Optional[date] = None
    start_time: time
    end_time: time
    description: Optional[str] = None
    days_of_week: DaysOfWeekCreate

    @field_validator("name")
    @classmethod
    def validate_name(cls, v: str) -> str:
        return _strip_and_reject_blank(v, "name")

    @field_validator("task_id", mode="before")
    @classmethod
    def validate_task_id(cls, v):
        return _strip_to_none(v)

    @field_validator("description", mode="before")
    @classmethod
    def validate_description(cls, v):
        return _strip_to_none(v)

    @model_validator(mode="after")
    def validate_schedule_range(self):
        if self.start_time >= self.end_time:
            raise ValueError("start_time must be before end_time")

        if self.start_date is not None and self.end_date is not None:
            if self.start_date > self.end_date:
                raise ValueError("start_date must be on or before end_date")

        return self


class ScheduleUpdate(BaseModel):
    """
    [요청] PUT /schedules/{schedule_id}

    현재 구현에서는 선택 필드만 부분 업데이트하는 방식으로 동작합니다.

    클라이언트 입력 형식:
    - start_time, end_time: HH:MM[:SS]
    - start_date, end_date: YYYY-MM-DD
    """
    model_config = ConfigDict(str_strip_whitespace=True)

    task_id: Optional[str] = None
    name: Optional[str] = None
    start_date: Optional[date] = None
    end_date: Optional[date] = None
    description: Optional[str] = None
    start_time: Optional[time] = None
    end_time: Optional[time] = None
    days_of_week: Optional[DaysOfWeekUpdate] = None
    is_active: Optional[bool] = None

    @field_validator("name", mode="before")
    @classmethod
    def validate_name(cls, v):
        if v is None:
            return None
        return _strip_and_reject_blank(v, "name")

    @field_validator("task_id", mode="before")
    @classmethod
    def validate_task_id(cls, v):
        return _strip_to_none(v)

    @field_validator("description", mode="before")
    @classmethod
    def validate_description(cls, v):
        return _strip_to_none(v)

    @model_validator(mode="after")
    def validate_schedule_range(self):
        if self.start_time is not None and self.end_time is not None:
            if self.start_time >= self.end_time:
                raise ValueError("start_time must be before end_time")

        if self.start_date is not None and self.end_date is not None:
            if self.start_date > self.end_date:
                raise ValueError("start_date must be on or before end_date")

        return self


class ScheduleRead(BaseModel):
    """
    [응답] GET /schedules, POST /schedules 등 조회/생성 성공 시
    데이터베이스에 저장된 스케줄 정보를 클라이언트에게 반환할 때의 데이터 구조입니다.
    """
    id: str
    user_id: str
    task_id: Optional[str] = None
    name: str
    start_date: Optional[date] = None
    end_date: Optional[date] = None
    description: Optional[str] = None
    start_time: time
    end_time: time
    days_of_week: list[int]
    created_at: datetime
    is_active: bool

    model_config = {
        "from_attributes": True
    }