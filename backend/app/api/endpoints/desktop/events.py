from fastapi import APIRouter, HTTPException, Depends
from typing import List
from app.db import mongo
from app.models.event import EventInDB
from app.schemas.event import EventBatchCreate, EventCreateResponse
from app.api import deps
import uuid

router = APIRouter()

# --------------------------------------------------------------------------
# 이벤트 배치 업로드 (POST /api/v1/events/batch)
# --------------------------------------------------------------------------
@router.post("/batch", response_model=EventCreateResponse)
async def create_events_batch(
    batch: EventBatchCreate,
    # [핵심 수정] 토큰을 검증하고 user_id를 추출하여 주입받음
    user_id: str = Depends(deps.get_current_user_id) 
):
    if mongo.db is None:
        raise HTTPException(status_code=500, detail="Database connection failed")

    # 1. 데이터가 비어있는지 확인
    if not batch.events:
        return EventCreateResponse(status="success", count=0)

    # 2. 저장할 문서 리스트 준비
    documents = []

    for event_data in batch.events:
        # Pydantic 모델(EventCreate) -> DB 모델(EventInDB) 변환
        event_doc = EventInDB(
            id=str(uuid.uuid4()),
            user_id=user_id, # [수정] 진짜 user_id 사용
            session_id=event_data.session_id,
            client_event_id=event_data.client_event_id,
            timestamp=event_data.timestamp,
            app_name=event_data.app_name,
            window_title=event_data.window_title,
            activity_vector=event_data.activity_vector
        )
        
        # model_dump(by_alias=True)를 통해 id -> _id 매핑
        documents.append(event_doc.model_dump(by_alias=True))

    # 3. MongoDB에 일괄 저장 (Bulk Insert)
    if documents:
        result = await mongo.db.events.insert_many(documents)
        inserted_count = len(result.inserted_ids)
        print(f"Synced {inserted_count} events from desktop.")
        return EventCreateResponse(status="success", count=inserted_count)
    
    return EventCreateResponse(status="success", count=0)