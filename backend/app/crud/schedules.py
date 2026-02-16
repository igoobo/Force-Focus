# backend/app/crud/schedules.py

from bson import ObjectId
from bson.errors import InvalidId
from datetime import datetime
from fastapi import HTTPException

from app.schemas.schedule import ScheduleCreate, ScheduleUpdate, ScheduleRead


def get_schedules_collection():
    from app.db.mongo import db
    if db is None:
        raise RuntimeError("MongoDB not initialized. Did you call connect_to_mongo()?")
    return db["schedules"]


def _safe_object_id(schedule_id: str) -> ObjectId:
    # ✅ 공백 방지 안전망
    if isinstance(schedule_id, str):
        schedule_id = schedule_id.strip()

    try:
        return ObjectId(schedule_id)
    except (InvalidId, TypeError):
        raise HTTPException(status_code=400, detail="Invalid schedule_id")


def serialize_schedule(schedule) -> ScheduleRead:
    # start_time/end_time: "HH:MM:SS" -> time
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

    # ✅ 추가: start_date/end_date: "YYYY-MM-DD" -> date
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

        # ✅ 추가
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
    start_time/end_time이 time 객체로 들어오면 DB 저장용 문자열로 변환.
    start_date/end_date가 date 객체로 들어오면 DB 저장용 문자열로 변환.
    (create_schedule에서 문자열로 저장하고 있어서 형식 통일)
    """
    if "start_time" in update_fields and update_fields["start_time"] is not None:
        st = update_fields["start_time"]
        try:
            update_fields["start_time"] = st.strftime("%H:%M:%S")
        except Exception:
            pass

    if "end_time" in update_fields and update_fields["end_time"] is not None:
        et = update_fields["end_time"]
        try:
            update_fields["end_time"] = et.strftime("%H:%M:%S")
        except Exception:
            pass

    # ✅ 추가: date -> "YYYY-MM-DD"
    if "start_date" in update_fields and update_fields["start_date"] is not None:
        sd = update_fields["start_date"]
        try:
            update_fields["start_date"] = sd.strftime("%Y-%m-%d")
        except Exception:
            pass

    if "end_date" in update_fields and update_fields["end_date"] is not None:
        ed = update_fields["end_date"]
        try:
            update_fields["end_date"] = ed.strftime("%Y-%m-%d")
        except Exception:
            pass

    return update_fields


# CREATE
async def create_schedule(user_id: str, schedule_data: ScheduleCreate) -> ScheduleRead:
    schedules_collection = get_schedules_collection()
    new_schedule = {
        "user_id": user_id,
        "name": schedule_data.name,
        "task_id": schedule_data.task_id,

        # ✅ 추가: date는 문자열로 저장 (없으면 None/미저장 가능)
        "start_date": schedule_data.start_date.strftime("%Y-%m-%d") if schedule_data.start_date else None,
        "end_date": schedule_data.end_date.strftime("%Y-%m-%d") if schedule_data.end_date else None,
        "description": schedule_data.description,

        "start_time": schedule_data.start_time.strftime("%H:%M:%S"),  # 문자열로 변환
        "end_time": schedule_data.end_time.strftime("%H:%M:%S"),      # 문자열로 변환
        "days_of_week": schedule_data.days_of_week,
        "created_at": datetime.now(),
        "is_active": True,
    }
    result = await schedules_collection.insert_one(new_schedule)
    saved = await schedules_collection.find_one({"_id": result.inserted_id})
    if not saved:
        raise HTTPException(status_code=500, detail="Failed to create schedule")
    return serialize_schedule(saved)


# READ ALL
async def get_schedules(user_id: str):
    schedules_collection = get_schedules_collection()
    cursor = schedules_collection.find({"user_id": user_id})
    return [serialize_schedule(doc) async for doc in cursor]


# READ ONE
async def get_schedule(user_id: str, schedule_id: str):
    schedules_collection = get_schedules_collection()
    oid = _safe_object_id(schedule_id)
    doc = await schedules_collection.find_one({"_id": oid, "user_id": user_id})
    return serialize_schedule(doc) if doc else None


# UPDATE
async def update_schedule(user_id: str, schedule_id: str, schedule_data: ScheduleUpdate):
    schedules_collection = get_schedules_collection()
    oid = _safe_object_id(schedule_id)

    update_fields = {k: v for k, v in schedule_data.model_dump().items() if v is not None}
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


# DELETE
async def delete_schedule(user_id: str, schedule_id: str) -> bool:
    schedules_collection = get_schedules_collection()
    oid = _safe_object_id(schedule_id)
    result = await schedules_collection.delete_one({"_id": oid, "user_id": user_id})
    return result.deleted_count == 1
