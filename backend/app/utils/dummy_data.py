import uuid
from datetime import datetime, timedelta
from pydantic import BaseModel
from typing import Optional
from app.crud import sessions as session_crud
from app.schemas.session import SessionCreate

# update_session 내부에서 참조하는 모든 필드를 포함하도록 확장
class SessionUpdate(BaseModel):
    task_id: Optional[str] = None
    start_time: Optional[datetime] = None
    goal_duration: Optional[float] = None
    end_time: Optional[datetime] = None
    status: Optional[str] = None
    interruption_count: Optional[int] = None

async def create_dummy_sessions(user_id: str):
    """분석을 위한 가짜 세션 데이터 3개 생성"""
    now = datetime.utcnow()
    dummy_data = [
        {"task_id": "백엔드 개발 및 API 설계", "goal": 30.0, "offset": 5, "duration": 1800, "interrupts": 1},
        {"task_id": "자료 조사 및 웹 서핑", "goal": 45.0, "offset": 3, "duration": 3600, "interrupts": 8},
        {"task_id": "문서 작성 및 정리", "goal": 60.0, "offset": 1, "duration": 3600, "interrupts": 2},
    ]

    for data in dummy_data:
        # 1. 세션 생성용 페이로드 구성
        payload = SessionCreate(
            task_id=data["task_id"],
            start_time=now - timedelta(hours=data["offset"]),
            goal_duration=data["goal"]
        )
        
        # 2. CRUD를 통해 DB에 초기 데이터 삽입
        session = await session_crud.start_session(user_id, payload)
        
        # 3. update_session이 내부적으로 참조하는 모든 속성을 객체 형태로 구성
        update_payload = SessionUpdate(
            task_id=data["task_id"],
            start_time=now - timedelta(hours=data["offset"]),
            goal_duration=data["goal"],
            end_time=now - timedelta(hours=data["offset"]) + timedelta(seconds=data["duration"]),
            status="completed",
            interruption_count=data["interrupts"]
        )
        
        # 4. 업데이트 수행
        await session_crud.update_session(user_id, session.id, update_payload)