# 파일 위치: backend/app/schemas/session.py

from pydantic import BaseModel
from datetime import datetime
from typing import Optional

# --- API 요청(Request) 스키마 ---

class SessionCreate(BaseModel):
    """
    [요청] POST /sessions/start
    새로운 집중 세션을 시작할 때 클라이언트가 보내는 데이터 구조입니다.
    """
    task_id: Optional[str] = None
    start_time: datetime
    goal_duration: Optional[float] = None # 목표 집중 시간 (분 단위)

class SessionUpdate(BaseModel):
    """
    [요청] PUT /sessions/{session_id}
    진행 중인 세션 정보를 업데이트할 때 사용합니다. 세션 종료 시 'end_time'과 'status'를 보냅니다.
    """
    end_time: Optional[datetime] = None
    status: Optional[str] = None # "completed", "cancelled" 등
    goal_duration: Optional[float] = None
    interruption_count: Optional[int] = None

# --- API 응답(Response) 스키마 ---

class SessionRead(BaseModel):
    """
    [응답] 세션 정보를 클라이언트에게 반환할 때의 데이터 구조입니다.
    (예: GET /sessions/current, POST /sessions/start 성공 시)
    """
    id: str
    user_id: str
    task_id: Optional[str] = None
    start_time: datetime
    end_time: Optional[datetime] = None
    duration: Optional[float] = None # 세션 종료 후 계산된 시간 (초 단위)
    status: str
    goal_duration: Optional[float] = None
    interruption_count: int

    class Config:
        orm_mode = True