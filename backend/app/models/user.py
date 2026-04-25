# backend/app/models/user.py

from datetime import datetime, timezone
from typing import Optional, List, Dict, Any

from pydantic import BaseModel, Field, EmailStr
from bson import ObjectId

from app.models.common import PyObjectId


class UserInDB(BaseModel):
    """
    User 공통 도메인 모델

    ✔ 플랫폼 독립 (web / mobile / desktop 공용)
    ✔ 현재 시스템은 Google 로그인 기반 사용자 식별을 사용
    ✔ 외부 응답에서는 _id 대신 문자열 id를 사용
    """

    id: PyObjectId = Field(default_factory=ObjectId, alias="_id")
    email: EmailStr
    google_id: str

    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    last_login_at: Optional[datetime] = None

    settings: Dict[str, Any] = Field(default_factory=dict)
    fcm_tokens: List[str] = Field(default_factory=list)
    blocked_apps: List[str] = Field(default_factory=list)

    model_config = {
        "populate_by_name": True,
        "arbitrary_types_allowed": True,
        "from_attributes": True,
    }