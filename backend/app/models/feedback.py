# backend/app/models/feedback.py

from datetime import datetime
from enum import Enum
from typing import Optional

from pydantic import BaseModel, Field
from bson import ObjectId

from app.models.common import PyObjectId


class FeedbackTypeEnum(str, Enum):
    IS_WORK = "is_work"
    DISTRACTION_IGNORED = "distraction_ignored"


class FeedbackInDB(BaseModel):
    """
    MongoDB의 'user_feedback' 컬렉션에 저장되는 완전한 형태의 데이터 모델입니다.
    """
    id: PyObjectId = Field(default_factory=ObjectId, alias="_id")
    user_id: str
    client_event_id: Optional[str] = None

    # "is_work", "distraction_ignored" 등
    feedback_type: FeedbackTypeEnum

    timestamp: datetime

    model_config = {
        "populate_by_name": True,
        "arbitrary_types_allowed": True,
        "from_attributes": True,
    }
