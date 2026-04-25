# backend/app/models/task.py

from pydantic import BaseModel, Field
from datetime import datetime, timezone
from typing import Optional

from bson import ObjectId
from app.models.common import PyObjectId


class TaskInDB(BaseModel):
    """
    MongoDB의 'tasks' 컬렉션에 저장되는 완전한 형태의 데이터 모델입니다.
    created_at은 timezone-aware UTC 기준으로 저장합니다.
    """
    id: PyObjectId = Field(default_factory=ObjectId, alias="_id")
    user_id: str
    name: str
    description: Optional[str] = None
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    due_date: Optional[datetime] = None
    status: str = "pending"
    linked_session_id: Optional[str] = None
    target_executable: Optional[str] = None
    target_arguments: Optional[str] = None
    isCustom: bool = True

    model_config = {
        "populate_by_name": True,
        "from_attributes": True,
        "arbitrary_types_allowed": True,
    }