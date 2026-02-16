from typing import List, Any
from fastapi import APIRouter, Depends, status, BackgroundTasks

# [핵심 수정] crud.feedback 모듈을 직접 import하여 별칭(alias) 사용
from app.crud import feedback as crud_feedback
from app.api import deps
# User 모델 import 제거 (불필요)
from app.schemas.feedback import FeedbackCreate

router = APIRouter()

@router.post("/batch", status_code=status.HTTP_201_CREATED)
async def receive_feedback_batch(
    feedbacks: List[FeedbackCreate],
    background_tasks: BackgroundTasks,
    # User 객체 대신 Token에서 ID(str)만 직접 수신
    user_id: str = Depends(deps.get_current_user_id),
) -> Any:
    """
    [Desktop Agent] 피드백 로그 배치 수신
    """
    saved_count = 0
    
    for feedback_in in feedbacks:
        # [수정] crud.feedback -> crud_feedback 으로 호출 변경
        await crud_feedback.create_feedback(
            user_id=user_id,
            data=feedback_in
        )
        saved_count += 1
    
    return {
        "status": "success", 
        "received": len(feedbacks), 
        "saved": saved_count
    }
