# backend/app/crud/feedback.py

from typing import List, Optional

from bson import ObjectId
from bson.errors import InvalidId
from fastapi import HTTPException

from app.db.mongo import get_db
from app.schemas.feedback import FeedbackCreate, FeedbackRead, FeedbackTypeEnum


def get_feedback_collection():
    """
    Motor DB 핸들에서 user_feedback 컬렉션을 가져옵니다.
    """
    return get_db()["user_feedback"]


def _safe_object_id(feedback_id: str) -> ObjectId:
    try:
        return ObjectId(feedback_id)
    except (InvalidId, TypeError):
        raise HTTPException(status_code=400, detail="Invalid feedback_id")


def serialize_feedback_read(doc) -> FeedbackRead:
    """
    Mongo document(dict) -> FeedbackRead (응답용)
    핵심: id는 반드시 str로 변환
    """
    return FeedbackRead(
        id=str(doc["_id"]),
        user_id=doc["user_id"],
        event_id=doc.get("event_id"),
        client_event_id=doc.get("client_event_id"),
        feedback_type=doc["feedback_type"],  # DB에 str로 있어도 Enum으로 자동 캐스팅됨
        timestamp=doc["timestamp"],
    )


# CREATE
async def create_feedback(user_id: str, data: FeedbackCreate) -> FeedbackRead:
    col = get_feedback_collection()

    doc = {
        "user_id": user_id,
        "client_event_id": data.client_event_id,
        "feedback_type": data.feedback_type.value
        if isinstance(data.feedback_type, FeedbackTypeEnum)
        else str(data.feedback_type),
        "timestamp": data.timestamp,
    }

    res = await col.insert_one(doc)
    saved = await col.find_one({"_id": res.inserted_id})
    if not saved:
        raise HTTPException(status_code=500, detail="Failed to create feedback")

    return serialize_feedback_read(saved)


# READ ALL
async def get_feedbacks(
    user_id: str,
    event_id: Optional[str] = None,
    feedback_type: Optional[FeedbackTypeEnum] = None,
    limit: int = 50,
) -> List[FeedbackRead]:
    col = get_feedback_collection()

    q = {"user_id": user_id}
    if event_id:
        q["event_id"] = event_id
    if feedback_type:
        q["feedback_type"] = feedback_type.value if isinstance(feedback_type, FeedbackTypeEnum) else str(feedback_type)

    safe_limit = max(1, min(limit, 1000))
    cursor = col.find(q).sort("timestamp", -1).limit(safe_limit)
    docs = await cursor.to_list(length=safe_limit)

    return [serialize_feedback_read(d) for d in docs]


# READ ONE
async def get_feedback(feedback_id: str) -> Optional[FeedbackRead]:
    col = get_feedback_collection()
    oid = _safe_object_id(feedback_id)

    doc = await col.find_one({"_id": oid})
    if not doc:
        return None

    return serialize_feedback_read(doc)


# DELETE
async def delete_feedback(feedback_id: str) -> bool:
    col = get_feedback_collection()
    oid = _safe_object_id(feedback_id)

    res = await col.delete_one({"_id": oid})
    return res.deleted_count == 1
