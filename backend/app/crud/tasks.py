# backend/app/crud/tasks.py

from bson import ObjectId
from bson.errors import InvalidId
from datetime import datetime
from fastapi import HTTPException

from app.schemas.task import TaskCreate, TaskUpdate, TaskRead


def get_tasks_collection():
    from app.db.mongo import db
    if db is None:
        raise RuntimeError("MongoDB not initialized. Did you call connect_to_mongo()?")
    return db["tasks"]


def _safe_object_id(task_id: str) -> ObjectId:
    # ✅ 공백 방지 안전망 (leading/trailing space로 InvalidId 나는 케이스 방지)
    if isinstance(task_id, str):
        task_id = task_id.strip()

    try:
        return ObjectId(task_id)
    except (InvalidId, TypeError):
        raise HTTPException(status_code=400, detail="Invalid task_id")


def serialize_task(task) -> TaskRead:
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
        target_arguments=task.get("target_arguments"),
        # ✅ 추가
        isCustom=task.get("isCustom", False),
    )


# CREATE
async def create_task(user_id: str, task_data: TaskCreate) -> TaskRead:
    tasks_collection = get_tasks_collection()

    payload = task_data.model_dump()

    # ✅ payload에서 created_at/status가 들어올 여지를 차단(정책적으로 서버가 소유)
    payload.pop("created_at", None)
    payload.pop("status", None)

    new_task = {
        "user_id": user_id,
        **payload,
        # 방어: 혹시 payload에 안 왔어도 기본값 보장
        "isCustom": task_data.isCustom,
        "created_at": datetime.now(),
        "status": "pending",
    }

    result = await tasks_collection.insert_one(new_task)
    saved = await tasks_collection.find_one({"_id": result.inserted_id})
    if not saved:
        raise HTTPException(status_code=500, detail="Failed to create task")

    return serialize_task(saved)


# READ ALL
async def get_tasks(user_id: str):
    tasks_collection = get_tasks_collection()
    cursor = tasks_collection.find({"user_id": user_id})
    return [serialize_task(doc) async for doc in cursor]


# READ ONE
async def get_task(user_id: str, task_id: str):
    tasks_collection = get_tasks_collection()
    oid = _safe_object_id(task_id)
    doc = await tasks_collection.find_one({"_id": oid, "user_id": user_id})
    return serialize_task(doc) if doc else None


# UPDATE
async def update_task(user_id: str, task_id: str, task_data: TaskUpdate):
    tasks_collection = get_tasks_collection()
    oid = _safe_object_id(task_id)

    update_fields = {
        k: v for k, v in task_data.model_dump().items()
        if v is not None
    }

    if not update_fields:
        return await get_task(user_id, task_id)

    result = await tasks_collection.update_one(
        {"_id": oid, "user_id": user_id},
        {"$set": update_fields},
    )

    if result.matched_count == 0:
        return None  # endpoint에서 404 처리

    updated = await tasks_collection.find_one({"_id": oid, "user_id": user_id})
    return serialize_task(updated) if updated else None


# DELETE
async def delete_task(user_id: str, task_id: str) -> bool:
    tasks_collection = get_tasks_collection()
    oid = _safe_object_id(task_id)
    result = await tasks_collection.delete_one({"_id": oid, "user_id": user_id})
    return result.deleted_count == 1
