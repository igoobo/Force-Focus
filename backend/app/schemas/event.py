# backend/app/schemas/feeeventdback.py

from datetime import datetime
from typing import Dict, Optional, Any, List
from pydantic import BaseModel, Field


class EventCreate(BaseModel):
    """
    [요청] 단일 이벤트 생성
    - user_id는 (보통) JWT에서 뽑지만, 테스트/확장 대비로 Optional 허용
    """
    user_id: Optional[str] = None
    session_id: Optional[str] = None
    client_event_id: Optional[str] = None
    timestamp: datetime
    app_name: Optional[str] = None
    window_title: Optional[str] = None
    activity_vector: Dict[str, Any] = Field(default_factory=dict)


class EventBatchCreate(BaseModel):
    """
    [요청] 배치 이벤트 생성
    {
      "events": [ ... ]
    }
    """
    events: List[EventCreate]


class EventRead(BaseModel):
    """
    [응답] 이벤트 조회
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


# 배치 전송 요청 스키마
class EventBatchCreate(BaseModel):
    """
    [요청] POST /events/batch
    Rust의 EventBatchRequest 구조체와 매핑됩니다.
    {
        "events": [ ... ]
    }
    """
    events: List[EventCreate]

class EventCreateResponse(BaseModel):
    """
    [응답] 이벤트 생성/배치 생성 공용
    - 단일 생성: event_id
    - 배치 생성: count
    """
    status: str = "success"

    # 배치 처리 시에는 저장된 개수를 반환하는 것이 일반적
    count: Optional[int] = None
    event_id: Optional[str] = None

