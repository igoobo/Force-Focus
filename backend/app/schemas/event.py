# 파일 위치: backend/app/schemas/event.py

from pydantic import BaseModel
from datetime import datetime
from typing import Dict, Optional

class EventCreate(BaseModel):
    """
    [요청] POST /events
    데스크탑 에이전트가 서버로 전송하는 단일 활동 이벤트의 데이터 구조입니다.
    """
    session_id: str
    timestamp: datetime
    app_name: str
    window_title: str
    activity_vector: Dict[str, float] # 예: [활성창 전환 빈도, 키 입력 빈도, 유휴 상태, 마우스 활동, 클립보드 활동]


class EventCreateResponse(BaseModel):
    """
    [응답] POST /events 성공 시
    이벤트가 성공적으로 저장되었음을 알리는 응답 데이터 구조입니다.
    """
    status: str = "success"
    event_id: str