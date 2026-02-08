# backend/app/schemas/event.py

from datetime import datetime
from typing import Any, Dict, List, Optional

from pydantic import BaseModel, Field, ConfigDict, field_validator


# -------------------------
# 공백 방지 공통 유틸
# -------------------------
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


class EventCreate(BaseModel):
    """
    [요청] 단일 이벤트 생성
    - user_id는 (보통) JWT에서 뽑지만, 테스트/확장 대비로 Optional 허용
    """
    # ✅ 공통 strip 적용
    model_config = ConfigDict(str_strip_whitespace=True)

    user_id: Optional[str] = None
    session_id: Optional[str] = None
    timestamp: datetime
    app_name: Optional[str] = None
    window_title: Optional[str] = None
    activity_vector: Dict[str, Any] = Field(default_factory=dict)

    @field_validator("user_id", "session_id", "app_name", "window_title", mode="before")
    @classmethod
    def validate_optional_strings(cls, v):
        # Optional[str]는 "   " -> None으로 정규화 + strip 적용
        return _strip_to_none(v)


class EventBatchCreate(BaseModel):
    """
    [요청] POST /events/batch (배치 이벤트 생성)
    Rust의 EventBatchRequest 구조체와 매핑됩니다.
    {
      "events": [ ... ]
    }
    """
    # ✅ 공통 strip 적용
    model_config = ConfigDict(str_strip_whitespace=True)

    events: List[EventCreate] = Field(min_length=1)


class EventRead(BaseModel):
    """
    [응답] 이벤트 조회
    """
    id: str
    user_id: str
    session_id: Optional[str] = None
    timestamp: datetime
    app_name: Optional[str] = None
    window_title: Optional[str] = None
    activity_vector: Dict[str, Any] = Field(default_factory=dict)

    model_config = {"from_attributes": True}


class EventCreateResponse(BaseModel):
    """
    [응답] 이벤트 생성/배치 생성 공용
    - 단일 생성: event_id
    - 배치 생성: count
    """
    status: str = "success"
    count: Optional[int] = None
    event_id: Optional[str] = None
