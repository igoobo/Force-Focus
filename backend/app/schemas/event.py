# 파일 위치: backend/app/schemas/event.py

from pydantic import BaseModel
from datetime import datetime
from typing import Dict, Optional, Any, List

class EventCreate(BaseModel):
    """
    [요청] POST /events
    데스크탑 에이전트가 서버로 전송하는 단일 활동 이벤트의 데이터 구조입니다.
    """
    session_id: str
    timestamp: datetime
    app_name: str
    window_title: str
    activity_vector: Dict[str, Any] # 예: [활성창 전환 빈도, 키 입력 빈도, 유휴 상태, 마우스 활동, 클립보드 활동]

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
    [응답] POST /events 성공 시
    이벤트가 성공적으로 저장되었음을 알리는 응답 데이터 구조입니다.
    """
    status: str = "success"
    # 배치 처리 시에는 저장된 개수를 반환하는 것이 일반적
    count: Optional[int] = None
    event_id: Optional[str] = None