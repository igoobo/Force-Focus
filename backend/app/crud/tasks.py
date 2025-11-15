# backend/app/crud/tasks.py
from motor.motor_asyncio import AsyncIOMotorClient
from bson import ObjectId
from datetime import datetime

from app.models.task import TaskInDB
from app.schemas.task import TaskCreate, TaskUpdate, TaskRead

client = AsyncIOMotorClient("mongodb://localhost:27017")
db = client["scheduler_db"]
tasks_collection = db["tasks"]


def serialize_task(task) -> TaskRead:
    """MongoDB Document → Pydantic 모델 변환"""
    return TaskRead(
        id=str(task["_id"]),
        user_id=task["user_id"],
        name=task["name"],
        description=task.get("description"),
        created_at=task["created_at"],
        due_date=task.get("due_date"),
        status=task["status"],
        linked_session_id=task.get("linked_session_id"),
        target_executable=task.get("target_executable"),
        target_arguments=task.get("target_arguments")
    )


# CREATE
async def create_task(user_id: str, task_data: TaskCreate) -> TaskRead:
    new_task = {
        "user_id": user_id,
        **task_data.dict(),
        "created_at": datetime.now(),
        "status": "pending",
    }
    result = await tasks_collection.insert_one(new_task)
    saved = await tasks_collection.find_one({"_id": result.inserted_id})
    return serialize_task(saved)


# READ ALL
async def get_tasks(user_id: str):
    cursor = tasks_collection.find({"user_id": user_id})
    return [serialize_task(doc) async for doc in cursor]


# READ ONE
async def get_task(task_id: str):
    doc = await tasks_collection.find_one({"_id": ObjectId(task_id)})
    return serialize_task(doc) if doc else None


# UPDATE
async def update_task(task_id: str, task_data: TaskUpdate):
    update_fields = {k: v for k, v in task_data.dict().items() if v is not None}
    if not update_fields:
        return None

    result = await tasks_collection.update_one(
        {"_id": ObjectId(task_id)}, {"$set": update_fields}
    )

    if result.modified_count == 0:
        return None

    updated = await tasks_collection.find_one({"_id": ObjectId(task_id)})
    return serialize_task(updated)


# DELETE
async def delete_task(task_id: str) -> bool:
    result = await tasks_collection.delete_one({"_id": ObjectId(task_id)})
    return result.deleted_count == 1
