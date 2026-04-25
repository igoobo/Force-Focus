# backend/app/schemas/session.py

from datetime import datetime
from typing import Optional, Literal

from pydantic import BaseModel, Field, ConfigDict, field_validator


SessionStatus = Literal["active", "completed", "cancelled"]


def _strip_to_none(v):
    if v is None:
        return None
    if not isinstance(v, str):
        return v
    s = v.strip()
    return s or None


class SessionCreate(BaseModel):
    """
    [요청] POST /sessions/start
    새로운 집중 세션 시작

    클라이언트 계약:
    - id 필드는 client_session_id의 alias로 수신
    """
    model_config = ConfigDict(
        str_strip_whitespace=True,
        populate_by_name=True,
    )

    client_session_id: Optional[str] = Field(None, alias="id")
    task_id: Optional[str] = None
    start_time: Optional[datetime] = None
    goal_duration: Optional[float] = None  # 분 단위
    profile_id: Optional[str] = None

    @field_validator("client_session_id", "task_id", "profile_id", mode="before")
    @classmethod
    def normalize_optional_ids(cls, v):
        return _strip_to_none(v)

    @field_validator("goal_duration")
    @classmethod
    def validate_goal_duration(cls, v):
        if v is None:
            return None
        if v < 0:
            raise ValueError("goal_duration must be >= 0")
        return v


class SessionUpdate(BaseModel):
    """
    [요청] PUT /sessions/{session_id}
    세션 종료/상태 변경/메타데이터 갱신
    """
    model_config = ConfigDict(str_strip_whitespace=True)

    end_time: Optional[datetime] = None
    end_time_s: Optional[float] = None
    status: Optional[SessionStatus] = None
    goal_duration: Optional[float] = None
    interruption_count: Optional[int] = None

    @field_validator("goal_duration")
    @classmethod
    def validate_goal_duration(cls, v):
        if v is None:
            return None
        if v < 0:
            raise ValueError("goal_duration must be >= 0")
        return v

    @field_validator("interruption_count")
    @classmethod
    def validate_interruption_count(cls, v):
        if v is None:
            return None
        if v < 0:
            raise ValueError("interruption_count must be >= 0")
        return v

    @field_validator("end_time_s")
    @classmethod
    def validate_end_time_s(cls, v):
        if v is None:
            return None
        if v < 0:
            raise ValueError("end_time_s must be >= 0")
        return v


class SessionRead(BaseModel):
    """
    [응답] 세션 반환
    """
    model_config = ConfigDict(from_attributes=True, populate_by_name=True)

    id: str
    user_id: str
    task_id: Optional[str] = None
    profile_id: Optional[str] = None
    client_session_id: Optional[str] = None

    start_time: datetime
    end_time: Optional[datetime] = None
    duration: Optional[float] = None  # 초 단위
    status: SessionStatus
    goal_duration: Optional[float] = None
    interruption_count: int = Field(default=0)