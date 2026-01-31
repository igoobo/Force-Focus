import uuid
from datetime import datetime, timedelta
from app.crud import sessions as session_crud
from app.schemas.session import SessionCreate

async def create_dummy_sessions(user_id: str):
    """분석을 위한 가짜 세션 데이터 3개를 생성합니다."""
    now = datetime.utcnow()
    dummy_data = [
        {"task_id": "백엔드 개발 및 API 설계", "goal": 30.0, "offset": 5, "duration": 1800, "interrupts": 1},
        {"task_id": "자료 조사 및 웹 서핑", "goal": 45.0, "offset": 3, "duration": 3600, "interrupts": 8},
        {"task_id": "문서 작성 및 정리", "goal": 60.0, "offset": 1, "duration": 3600, "interrupts": 2},
    ]

    for data in dummy_data:
        payload = SessionCreate(
            task_id=data["task_id"],
            start_time=now - timedelta(hours=data["offset"]),
            goal_duration=data["goal"]
        )
        # CRUD를 통해 DB에 삽입 (Status 등을 completed로 업데이트하는 과정 필요)
        session = await session_crud.start_session(user_id, payload)
        # 테스트를 위해 종료 상태로 강제 업데이트
        await session_crud.update_session(user_id, session.id, {
            "end_time": now - timedelta(hours=data["offset"]) + timedelta(seconds=data["duration"]),
            "status": "completed",
            "interruption_count": data["interrupts"]
        })