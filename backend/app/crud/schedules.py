# backend/app/crud/schedules.py

from datetime import datetime, timezone

from bson import ObjectId
from bson.errors import InvalidId
from fastapi import HTTPException

from app.db.mongo import get_db
from app.schemas.schedule import ScheduleCreate, ScheduleUpdate, ScheduleRead


def get_schedules_collection():
    db = get_db()
    if db is None:
        raise RuntimeError("MongoDB not initialized. Did you call connect_to_mongo()?")
    return db["schedules"]


def _utcnow() -> datetime:
    return datetime.now(timezone.utc)


def _safe_object_id(schedule_id: str) -> ObjectId:
    if isinstance(schedule_id, str):
        schedule_id = schedule_id.strip()

    try:
        return ObjectId(schedule_id)
    except (InvalidId, TypeError):
        raise HTTPException(status_code=400, detail="Invalid schedule_id")


def _serialize_time_value(value):
    if value is None:
        return None
    if hasattr(value, "strftime"):
        return value.strftime("%H:%M:%S")
    return value


def _serialize_date_value(value):
    if value is None:
        return None
    if hasattr(value, "isoformat"):
        return value.isoformat()
    return value


def serialize_schedule(schedule) -> ScheduleRead:
    start_time = (
        datetime.strptime(schedule["start_time"], "%H:%M:%S").time()
        if schedule.get("start_time")
        else None
    )
    end_time = (
        datetime.strptime(schedule["end_time"], "%H:%M:%S").time()
        if schedule.get("end_time")
        else None
    )
    start_date = (
        datetime.strptime(schedule["start_date"], "%Y-%m-%d").date()
        if schedule.get("start_date")
        else None
    )
    end_date = (
        datetime.strptime(schedule["end_date"], "%Y-%m-%d").date()
        if schedule.get("end_date")
        else None
    )

    return ScheduleRead(
        id=str(schedule.get("_id")),
        user_id=schedule.get("user_id", ""),
        task_id=schedule.get("task_id"),
        name=schedule.get("name", ""),
        start_date=start_date,
        end_date=end_date,
        description=schedule.get("description"),
        start_time=start_time,
        end_time=end_time,
        days_of_week=schedule.get("days_of_week", []),
        created_at=schedule.get("created_at"),
        is_active=schedule.get("is_active", True),
    )


def _normalize_schedule_update_fields(update_fields: dict) -> dict:
    """
    time/date 객체를 DB 저장용 문자열로 변환합니다.
    현재 schedules 컬렉션은 시간/날짜를 문자열 포맷으로 저장합니다.
    """
    normalized = dict(update_fields)

    if "start_time" in normalized and normalized["start_time"] is not None:
        normalized["start_time"] = _serialize_time_value(normalized["start_time"])

    if "end_time" in normalized and normalized["end_time"] is not None:
        normalized["end_time"] = _serialize_time_value(normalized["end_time"])

    if "start_date" in normalized and normalized["start_date"] is not None:
        normalized["start_date"] = _serialize_date_value(normalized["start_date"])

    if "end_date" in normalized and normalized["end_date"] is not None:
        normalized["end_date"] = _serialize_date_value(normalized["end_date"])

    return normalized


async def create_schedule(user_id: str, schedule_data: ScheduleCreate) -> ScheduleRead:
    schedules_collection = get_schedules_collection()

    new_schedule = {
        "user_id": user_id,
        "task_id": schedule_data.task_id,
        "name": schedule_data.name,
        "start_date": _serialize_date_value(schedule_data.start_date),
        "end_date": _serialize_date_value(schedule_data.end_date),
        "description": schedule_data.description,
        "start_time": _serialize_time_value(schedule_data.start_time),
        "end_time": _serialize_time_value(schedule_data.end_time),
        "days_of_week": schedule_data.days_of_week,
        "created_at": _utcnow(),
        "is_active": True,
    }

    result = await schedules_collection.insert_one(new_schedule)
    saved = await schedules_collection.find_one({"_id": result.inserted_id})
    if not saved:
        raise HTTPException(status_code=500, detail="Failed to create schedule")

    return serialize_schedule(saved)


async def get_schedules(user_id: str):
    schedules_collection = get_schedules_collection()
    cursor = schedules_collection.find({"user_id": user_id}).sort("created_at", -1)
    return [serialize_schedule(doc) async for doc in cursor]


async def get_schedule(user_id: str, schedule_id: str):
    schedules_collection = get_schedules_collection()
    oid = _safe_object_id(schedule_id)
    doc = await schedules_collection.find_one({"_id": oid, "user_id": user_id})
    return serialize_schedule(doc) if doc else None


async def update_schedule(user_id: str, schedule_id: str, schedule_data: ScheduleUpdate):
    schedules_collection = get_schedules_collection()
    oid = _safe_object_id(schedule_id)

    update_fields = {
        k: v for k, v in schedule_data.model_dump().items()
        if v is not None
    }
    update_fields = _normalize_schedule_update_fields(update_fields)

    if not update_fields:
        return await get_schedule(user_id, schedule_id)

    result = await schedules_collection.update_one(
        {"_id": oid, "user_id": user_id},
        {"$set": update_fields},
    )

    if result.matched_count == 0:
        return None

    updated = await schedules_collection.find_one({"_id": oid, "user_id": user_id})
    return serialize_schedule(updated) if updated else None


async def delete_schedule(user_id: str, schedule_id: str) -> bool:
    schedules_collection = get_schedules_collection()
    oid = _safe_object_id(schedule_id)
    result = await schedules_collection.delete_one({"_id": oid, "user_id": user_id})
    return result.deleted_count == 1