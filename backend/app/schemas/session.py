# backend/app/schemas/session.py

from datetime import datetime
from typing import Optional

from pydantic import BaseModel, Field, ConfigDict, field_validator


# --- API 요청(Request) 스키마 ---

class SessionCreate(BaseModel):
    """
    [요청] POST /sessions/start
    새로운 집중 세션 시작
    """
    model_config = ConfigDict(str_strip_whitespace=True)

    task_id: Optional[str] = None
    start_time: datetime
    goal_duration: Optional[float] = None  # 목표 집중 시간 (분 단위)

    # ML 모델 도입전 실험적 필드
    profile_id: Optional[str] = None

    @field_validator("task_id", "profile_id", mode="before")
    @classmethod
    def normalize_optional_ids(cls, v):
        """
        Optional[str]에서:
        - None은 그대로
        - "   " -> "" -> None
        - 나머지는 strip된 문자열
        """
        if v is None:
            return None
        if not isinstance(v, str):
            return v
        s = v.strip()
        return s or None


class SessionUpdate(BaseModel):
    """
    [요청] PUT /sessions/{session_id}
    진행 중인 세션 업데이트 (종료 시 end_time/status 포함)
    """
    model_config = ConfigDict(str_strip_whitespace=True)

    end_time: Optional[datetime] = None
    status: Optional[str] = None  # "completed", "cancelled" 등
    goal_duration: Optional[float] = None
    interruption_count: Optional[int] = None

    @field_validator("status", mode="before")
    @classmethod
    def validate_status(cls, v):
        if v is None:
            return None
        if not isinstance(v, str):
            return v
        s = v.strip()
        if s == "":
            raise ValueError("status must not be blank")
        return s


# --- API 응답(Response) 스키마 ---

class SessionRead(BaseModel):
    """
    [응답] 세션 반환
    """
    model_config = ConfigDict(from_attributes=True)

    id: str
    user_id: str
    task_id: Optional[str] = None
    profile_id: Optional[str] = None

    start_time: datetime
    end_time: Optional[datetime] = None
    duration: Optional[float] = None  # 초 단위
    status: str
    goal_duration: Optional[float] = None
    interruption_count: int = Field(default=0)
