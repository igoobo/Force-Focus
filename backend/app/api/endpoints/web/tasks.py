# backend/app/api/endpoints/web/tasks.py

from fastapi import APIRouter, HTTPException, status, Depends
from typing import List

from app.schemas.task import TaskCreate, TaskUpdate, TaskRead
from app.crud import tasks as task_crud
from app.api.deps import get_current_user_id

router = APIRouter(prefix="/tasks", tags=["Tasks"])

# CREATE
@router.post("/", response_model=TaskRead, status_code=status.HTTP_201_CREATED)
async def create_task(
    task: TaskCreate, 
    user_id: str = Depends(get_current_user_id)):
    return await task_crud.create_task(user_id, task)

# READ ALL
@router.get("/", response_model=List[TaskRead])
async def read_tasks(user_id: str = Depends(get_current_user_id)):
    return await task_crud.get_tasks(user_id)

# READ ONE
@router.get("/{task_id}", response_model=TaskRead)
async def read_task(task_id: str):
    task = await task_crud.get_task(task_id)
    if not task:
        raise HTTPException(status_code=404, detail="Task not found")
    return task

# UPDATE
@router.put("/{task_id}", response_model=TaskRead)
async def update_task(
    task_id: str, 
    task: TaskUpdate,
    user_id: str = Depends(get_current_user_id)):
    updated = await task_crud.update_task(task_id, task, user_id=user_id)
    if not updated:
        raise HTTPException(status_code=404, detail="Task not found or no changes")
    return updated

# DELETE
@router.delete("/{task_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_task(task_id: str):
    deleted = await task_crud.delete_task(task_id)
    if not deleted:
        raise HTTPException(status_code=404, detail="Task not found")
    return None
