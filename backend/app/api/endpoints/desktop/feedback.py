from typing import List, Any
from fastapi import APIRouter, Depends, status, BackgroundTasks

from app import crud
from app.api import deps
from app.schemas.feedback import FeedbackCreate

router = APIRouter()

@router.post("/batch", status_code=status.HTTP_201_CREATED)
async def receive_feedback_batch(
    feedbacks: List[FeedbackCreate],
    background_tasks: BackgroundTasks,
    user_id: str = Depends(deps.get_current_user_id),
) -> Any:
    """
    [Desktop Agent] 피드백 로그 배치 수신 (Log Shipping)
    
    """
    saved_count = 0
    
    # 배치 데이터 처리
    for feedback_in in feedbacks:
        # User 모델의 id 속성 접근(current_user.id) 대신, user_id 문자열을 바로 사용
        await crud.feedback.create_feedback(
            user_id=user_id, 
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