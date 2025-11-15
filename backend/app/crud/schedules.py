# backend/app/crud/schedules.py
from motor.motor_asyncio import AsyncIOMotorClient
from bson import ObjectId
from app.models.schedule import ScheduleInDB
from app.schemas.schedule import ScheduleCreate, ScheduleUpdate, ScheduleRead
from datetime import datetime

client = AsyncIOMotorClient("mongodb://localhost:27017")
db = client["scheduler_db"]
schedules_collection = db["schedules"]

def serialize_schedule(schedule) -> ScheduleRead:
    """MongoDB Document → Pydantic 모델로 변환"""
    return ScheduleRead(
        id=str(schedule["_id"]),
        user_id=schedule["user_id"],
        task_id=schedule.get("task_id"),
        name=schedule["name"],
        start_time=schedule["start_time"],
        end_time=schedule["end_time"],
        days_of_week=schedule["days_of_week"],
        created_at=schedule["created_at"],
        is_active=schedule["is_active"]
    )

# CREATE
async def create_schedule(user_id: str, schedule_data: ScheduleCreate) -> ScheduleRead:
    new_schedule = {
        "user_id": user_id,
        **schedule_data.dict(),
        "created_at": datetime.now(),
        "is_active": True
    }
    result = await schedules_collection.insert_one(new_schedule)
    saved = await schedules_collection.find_one({"_id": result.inserted_id})
    return serialize_schedule(saved)

# READ (전체)
async def get_schedules(user_id: str):
    cursor = schedules_collection.find({"user_id": user_id})
    return [serialize_schedule(doc) async for doc in cursor]

# READ (단일)
async def get_schedule(schedule_id: str):
    doc = await schedules_collection.find_one({"_id": ObjectId(schedule_id)})
    return serialize_schedule(doc) if doc else None

# UPDATE
async def update_schedule(schedule_id: str, schedule_data: ScheduleUpdate):
    update_fields = {k: v for k, v in schedule_data.dict().items() if v is not None}
    result = await schedules_collection.update_one(
        {"_id": ObjectId(schedule_id)},
        {"$set": update_fields}
    )
    if result.modified_count == 0:
        return None
    updated = await schedules_collection.find_one({"_id": ObjectId(schedule_id)})
    return serialize_schedule(updated)

# DELETE
async def delete_schedule(schedule_id: str) -> bool:
    result = await schedules_collection.delete_one({"_id": ObjectId(schedule_id)})
    return result.deleted_count == 1
