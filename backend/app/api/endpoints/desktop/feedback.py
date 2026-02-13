from typing import List, Any
from fastapi import APIRouter, Depends, status, BackgroundTasks

from app import crud
from app.api import deps
from app.schemas.feedback import FeedbackCreate

from app.ml.train import train_user_model

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
    
    
    # 2. 백그라운드 학습 트리거
    # 즉시 응답을 반환하기 위해, 학습 작업은 백그라운드 큐에 등록만 합니다.
    # 정책: 피드백이 1개라도 들어오면 모델을 최신화합니다. (추후 '10개당 1번' 등으로 최적화 가능)
    if saved_count > 0:
        print(f"[Trigger] Scheduling training for user: {user_id}")
        background_tasks.add_task(train_user_model, user_id)

    return {
        "status": "success", 
        "received": len(feedbacks), 
        "saved": saved_count
    }