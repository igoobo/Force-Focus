# backend/app/schemas/event.py

from datetime import datetime, timezone
from typing import Any, Dict, List, Optional

from pydantic import BaseModel, Field, ConfigDict, field_validator


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


class EventCreate(BaseModel):
    """
    [요청] 단일 이벤트 생성

    web API 계약:
    - user_id는 인증 토큰에서 결정되므로 요청 본문에서 받지 않음
    - session_id는 문자열 식별자 계약을 따름
    - client_event_id는 클라이언트 측 이벤트 식별자(옵션)
    - activity_vector는 자유 dict를 허용하되, 클라이언트별 포맷 일관성을 유지해야 함
    """
    model_config = ConfigDict(str_strip_whitespace=True)

    session_id: Optional[str] = None
    client_event_id: Optional[str] = None
    timestamp: datetime
    app_name: Optional[str] = None
    window_title: Optional[str] = None
    activity_vector: Dict[str, Any] = Field(default_factory=dict)

    @field_validator("session_id", "client_event_id", "app_name", "window_title", mode="before")
    @classmethod
    def validate_optional_strings(cls, v):
        return _strip_to_none(v)
    @field_validator("timestamp", mode="before")
    @classmethod
    def validate_timestamp(cls, v):
        if v is None:
            return datetime.now(timezone.utc)

        if isinstance(v, str):
            v = v.replace("Z", "+00:00")
            dt = datetime.fromisoformat(v)
        else:
            dt = v

        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)

        return dt


class EventBatchCreate(BaseModel):
    """
    [요청] 배치 이벤트 생성용 스키마
    현재 web router에는 /events/batch 엔드포인트가 없으며,
    다른 클라이언트/에이전트 채널에서 사용할 수 있는 확장 스키마로 취급합니다.
    """
    model_config = ConfigDict(str_strip_whitespace=True)

    events: List[EventCreate] = Field(min_length=1)


class EventRead(BaseModel):
    """
    [응답] 이벤트 조회
    - id는 UUID 문자열 이벤트 식별자
    """
    id: str
    user_id: str
    session_id: Optional[str] = None
    client_event_id: Optional[str] = None
    timestamp: datetime
    app_name: Optional[str] = None
    window_title: Optional[str] = None
    activity_vector: Dict[str, Any] = Field(default_factory=dict)

    model_config = {"from_attributes": True}


class EventCreateResponse(BaseModel):
    """
    [응답] 이벤트 생성 응답
    현재 web API에서는 단일 생성 응답으로 사용합니다.
    """
    status: str = "success"
    event_id: Optional[str] = None