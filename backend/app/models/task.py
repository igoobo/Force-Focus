# 파일 위치: backend/app/models/task.py

from pydantic import BaseModel, Field
from datetime import datetime
from typing import Optional

class TaskInDB(BaseModel):
    """
    MongoDB의 'tasks' 컬렉션에 저장되는 완전한 형태의 데이터 모델입니다.
    데이터 모델링 표의 모든 필드를 포함합니다.
    """
    id: str = Field(..., alias="_id") # MongoDB의 '_id'를 'id'로 매핑
    user_id: str
    name: str
    description: Optional[str] = None
    created_at: datetime = Field(default_factory=datetime.now)
    due_date: Optional[datetime] = None
    status: str = "pending" # pending, completed, cancelled 등
    linked_session_id: Optional[str] = None # 현재 연결된 세션 ID
    target_executable: Optional[str] = None # 예: "Code.exe", "chrome.exe"
    target_arguments: Optional[str] = None  # 예: "--profile-directory=Work"

    class Config:
        allow_population_by_field_name = True
        orm_mode = True