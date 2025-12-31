# 파일 위치: backend/app/models/event.py

from pydantic import BaseModel, Field, ConfigDict
from datetime import datetime
from typing import Dict, Optional, Any

class EventInDB(BaseModel):
    """
    MongoDB의 'events' 컬렉션에 저장되는 완전한 형태의 데이터 모델입니다.
    데이터 모델링 표의 모든 필드를 포함합니다.
    """
    id: str = Field(..., alias="_id")  # MongoDB의 '_id'를 'id'로 매핑
    user_id: str
    session_id: str
    timestamp: datetime
    app_name: str
    window_title: str

    
    # Rust에서 보내는 visible_windows(List), tokens(List) 등을 모두 수용하기 위해 Any 사용
    activity_vector: Dict[str, Any] 

    model_config = ConfigDict(
        populate_by_name=True,
        from_attributes=True,
        arbitrary_types_allowed=True 
    )