# backend/app/schemas/feedback.py

from datetime import datetime
from enum import Enum
from typing import Optional
from pydantic import BaseModel, ConfigDict, field_validator


class FeedbackTypeEnum(str, Enum):
    """피드백의 종류를 정의하는 Enum"""
    IS_WORK = "is_work"
    DISTRACTION_IGNORED = "distraction_ignored"


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


class FeedbackCreate(BaseModel):
    """
    [요청] POST /feedback
    사용자가 시스템의 개입에 대해 피드백을 제출할 때 보내는 데이터 구조입니다.
    """
    # ✅ 공통 strip 적용
    model_config = ConfigDict(str_strip_whitespace=True)

    # event_id: str
    client_event_id: Optional[str] = None
    feedback_type: FeedbackTypeEnum
    timestamp: datetime

    @field_validator("client_event_id")
    @classmethod
    def validate_event_id(cls, v: str) -> str:
        return _strip_and_reject_blank(v, "client_event_id")


class FeedbackRead(BaseModel):
    """
    [응답] GET /feedback, POST /feedback 성공 시 등
    """
    id: str
    user_id: str
    # event_id: str
    client_event_id: Optional[str] = None
    feedback_type: FeedbackTypeEnum
    timestamp: datetime

    model_config = {
        "from_attributes": True
    }


class FeedbackCreateResponse(BaseModel):
    """
    [응답] POST /feedback 성공 시 (간단 응답 원하면 사용)
    """
    status: str = "success"
    feedback_id: str
