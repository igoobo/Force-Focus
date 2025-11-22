# backend/app/crud/schedules.py
from bson import ObjectId
from datetime import datetime
from app.schemas.schedule import ScheduleCreate, ScheduleUpdate, ScheduleRead

def get_schedules_collection():
    from app.db.mongo import db
    return db["schedules"]

def serialize_schedule(schedule) -> ScheduleRead:
    return ScheduleRead(
        id=str(schedule.get("_id")),
        user_id=schedule.get("user_id", ""),
        task_id=schedule.get("task_id"),
        name=schedule.get("name", ""),
        start_time=datetime.strptime(schedule["start_time"], "%H:%M:%S").time() if schedule.get("start_time") else None,
        end_time=datetime.strptime(schedule["end_time"], "%H:%M:%S").time() if schedule.get("end_time") else None,
        days_of_week=schedule.get("days_of_week", []),
        created_at=schedule.get("created_at"),
        is_active=schedule.get("is_active", True)
    )


# CREATE
async def create_schedule(user_id: str, schedule_data: ScheduleCreate) -> ScheduleRead:
    schedules_collection = get_schedules_collection()
    new_schedule = {
        "user_id": user_id,
        "name": schedule_data.name,
        "task_id": schedule_data.task_id,
        "start_time": schedule_data.start_time.strftime("%H:%M:%S"),  # 문자열로 변환
        "end_time": schedule_data.end_time.strftime("%H:%M:%S"),      # 문자열로 변환
        "days_of_week": schedule_data.days_of_week,
        "created_at": datetime.now(),
        "is_active": True
    }
    result = await schedules_collection.insert_one(new_schedule)
    saved = await schedules_collection.find_one({"_id": result.inserted_id})
    return serialize_schedule(saved)


# READ ALL
async def get_schedules(user_id: str):
    schedules_collection = get_schedules_collection()
    cursor = schedules_collection.find({"user_id": user_id})
    return [serialize_schedule(doc) async for doc in cursor]

# READ ONE
async def get_schedule(schedule_id: str):
    schedules_collection = get_schedules_collection()
    doc = await schedules_collection.find_one({"_id": ObjectId(schedule_id)})
    return serialize_schedule(doc) if doc else None

# UPDATE
async def update_schedule(schedule_id: str, schedule_data: ScheduleUpdate):
    schedules_collection = get_schedules_collection()
    update_fields = {k: v for k, v in schedule_data.dict().items() if v is not None}
    if not update_fields:
        return await get_schedule(schedule_id)
    await schedules_collection.update_one(
        {"_id": ObjectId(schedule_id)},
        {"$set": update_fields}
    )
    updated = await schedules_collection.find_one({"_id": ObjectId(schedule_id)})
    return serialize_schedule(updated)

# DELETE
async def delete_schedule(schedule_id: str) -> bool:
    schedules_collection = get_schedules_collection()
    result = await schedules_collection.delete_one({"_id": ObjectId(schedule_id)})
    return result.deleted_count == 1
