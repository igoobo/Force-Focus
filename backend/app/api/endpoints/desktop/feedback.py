from typing import List, Any
from fastapi import APIRouter, Depends, status, BackgroundTasks

from app import crud
from app.api import deps
from app.models.user import User
from app.schemas.feedback import FeedbackCreate

router = APIRouter()

@router.post("/batch", status_code=status.HTTP_201_CREATED)
async def receive_feedback_batch(
    feedbacks: List[FeedbackCreate],
    background_tasks: BackgroundTasks,
    current_user: User = Depends(deps.get_current_user),
) -> Any:
    """
    [Desktop Agent] 피드백 로그 배치 수신 (Log Shipping)
    
    - 데스크탑 에이전트로부터 'FeedbackCreate' 리스트를 수신합니다.
    - 확인된 crud.feedback.create_feedback 함수를 사용하여 DB에 저장합니다.
    """
    saved_count = 0
    
    # 배치 데이터 처리
    for feedback_in in feedbacks:
        # user_id와 data 객체를 인자로 전달
        await crud.feedback.create_feedback(
            user_id=str(current_user.id),
            data=feedback_in
        )
        saved_count += 1
    
    # TODO: 추후 이곳에 재학습 트리거(Background Task) 로직이 추가될 예정입니다.
    # background_tasks.add_task(...)

    return {
        "status": "success", 
        "received": len(feedbacks), 
        "saved": saved_count
    }