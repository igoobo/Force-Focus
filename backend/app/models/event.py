# backend/app/models/event.py

from datetime import datetime
from typing import Dict, Optional, Any

from pydantic import BaseModel, Field, field_validator
from bson import ObjectId


class EventInDB(BaseModel):
    """
    MongoDB의 'events' 컬렉션에 저장되는 완전한 형태의 데이터 모델입니다.
    - events._id 는 UUID 문자열을 사용합니다.
    - 혹시 기존 데이터가 ObjectId여도 읽을 때 str로 변환합니다.
    """
    id: str = Field(..., alias="_id")
    user_id: str
    session_id: Optional[str] = None
    client_event_id: Optional[str] = None
    timestamp: datetime
    app_name: Optional[str] = None
    window_title: Optional[str] = None
    activity_vector: Dict[str, Any] = Field(default_factory=dict)

    @field_validator("id", mode="before")
    @classmethod
    def coerce_objectid_to_str(cls, v):
        if isinstance(v, ObjectId):
            return str(v)
        return v

    model_config = {
        "populate_by_name": True,
        "arbitrary_types_allowed": True,
        "from_attributes": True,
    }