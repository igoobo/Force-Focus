# 파일 위치: backend/app/models/event.py

from pydantic import BaseModel, Field
from datetime import datetime
from typing import Dict, Optional

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
    activity_vector: Dict[str, float] # List[float] -> Dict[str, float] # 예: [활성창 전환 빈도, 키 입력 빈도, 유휴 상태, 마우스 활동, 클립보드 활동 등]

    class Config:
        allow_population_by_field_name = True # '_id' 필드명으로도 데이터 채우기 허용
        orm_mode = True