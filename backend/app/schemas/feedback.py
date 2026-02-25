from datetime import datetime
from enum import Enum

# [수정] Field 추가 Import
from pydantic import BaseModel, ConfigDict, Field


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
    """
    # 1. 공백 자동 제거 설정
    model_config = ConfigDict(str_strip_whitespace=True)

    # 2. Pydantic 기본 기능(Field) 사용
    # min_length=1 : 공백 제거 후 빈 문자열이면 에러 발생시킴
    client_event_id: str = Field(..., min_length=1, description="Client generated event UUID")
    
    feedback_type: FeedbackTypeEnum
    timestamp: datetime

    @field_validator("client_event_id")
    @classmethod
    def validate_event_id(cls, v: str) -> str:
        return _strip_and_reject_blank(v, "client_event_id")


class FeedbackRead(BaseModel):
    """
    [응답] DB 조회 결과
    """
    id: str
    user_id: str
    client_event_id: str
    feedback_type: FeedbackTypeEnum
    timestamp: datetime

    model_config = ConfigDict(from_attributes=True)


class FeedbackCreateResponse(BaseModel):
    status: str = "success"
    feedback_id: str