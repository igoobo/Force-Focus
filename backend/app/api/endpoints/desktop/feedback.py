from typing import List, Any
from fastapi import APIRouter, Depends, status, BackgroundTasks, HTTPException

from app.db import mongo
from app.api import deps
from app.schemas.feedback import FeedbackCreate
from app.models.feedback import FeedbackInDB

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
    if mongo.db is None:
        raise HTTPException(status_code=500, detail="Database connection failed")
        
    saved_count = 0
    
    # 1. 데이터가 비어있는지 확인
    if not feedbacks:
        return {"status": "success", "received": 0, "saved": 0}

    # 2. 저장할 문서 리스트 준비
    documents = []
    
    # 배치 데이터 처리
    for feedback_in in feedbacks:
        feedback_doc = FeedbackInDB(
            user_id=user_id, 
            client_event_id=feedback_in.client_event_id,
            feedback_type=feedback_in.feedback_type,
            timestamp=feedback_in.timestamp
        )
        
        doc_dict = feedback_doc.model_dump(by_alias=True)
        # Enum 값을 문자열로 변환 (DB 저장 호환성)
        if hasattr(doc_dict.get("feedback_type"), "value"):
            doc_dict["feedback_type"] = doc_dict["feedback_type"].value
            
        documents.append(doc_dict)
    
    # 3. MongoDB에 일괄 저장 (Bulk Insert)
    if documents:
        result = await mongo.db.user_feedback.insert_many(documents)
        saved_count = len(result.inserted_ids)
        print(f"Synced {saved_count} feedbacks from desktop.")
    
    # 4. 백그라운드 학습 트리거
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
