# 파일 위치: backend/app/schemas/task.py

from pydantic import BaseModel
from datetime import datetime
from typing import Optional

# --- API 요청(Request) 스키마 ---
class TaskCreate(BaseModel):
    """
    [요청] POST /tasks
    새로운 할 일을 생성할 때 클라이언트가 보내는 데이터 구조입니다.
    user_id는 인증 토큰에서 가져오므로 클라이언트가 보낼 필요가 없습니다.
    """
    name: str
    description: Optional[str] = None
    due_date: Optional[datetime] = None
    target_executable: Optional[str] = None
    target_arguments: Optional[str] = None

class TaskUpdate(BaseModel):
    """
    [요청] PUT /tasks/{task_id}
    기존 할 일 정보를 업데이트할 때 클라이언트가 보내는 데이터 구조입니다.
    모든 필드는 선택 사항(Optional)입니다.
    """
    name: Optional[str] = None
    description: Optional[str] = None
    due_date: Optional[datetime] = None
    status: Optional[str] = None # "pending", "completed" 등으로 변경
    linked_session_id: Optional[str] = None
    target_executable: Optional[str] = None
    target_arguments: Optional[str] = None

# --- API 응답(Response) 스키마 ---
class TaskRead(BaseModel):
    """
    [응답] GET /tasks/{task_id}, POST /tasks 등 조회/생성 성공 시
    데이터베이스에 저장된 할 일 정보를 클라이언트에게 반환할 때의 데이터 구조입니다.
    """
    id: str
    user_id: str
    name: str
    description: Optional[str] = None
    created_at: datetime
    due_date: Optional[datetime] = None
    status: str
    linked_session_id: Optional[str] = None
    target_executable: Optional[str] = None
    target_arguments: Optional[str] = None

    class Config:
        orm_mode = True