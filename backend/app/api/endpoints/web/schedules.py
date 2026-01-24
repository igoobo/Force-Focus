# backend/app/api/endpoints/web/schedules.py

from fastapi import APIRouter, HTTPException, Depends, status
from typing import List

from app.schemas.schedule import ScheduleCreate, ScheduleUpdate, ScheduleRead
from app.crud import schedules as schedule_crud
from app.api.deps import get_current_user_id

router = APIRouter(prefix="/schedules", tags=["Schedules"])

# CREATE
@router.post("/", response_model=ScheduleRead, status_code=status.HTTP_201_CREATED)
async def create_schedule(
    schedule: ScheduleCreate,
    user_id: str = Depends(get_current_user_id)):
    return await schedule_crud.create_schedule(user_id, schedule)

# READ ALL
@router.get("/", response_model=List[ScheduleRead])
async def read_schedules(user_id: str = Depends(get_current_user_id)):
    return await schedule_crud.get_schedules(user_id)

# READ ONE
@router.get("/{schedule_id}", response_model=ScheduleRead)
async def read_schedule(schedule_id: str):
    schedule = await schedule_crud.get_schedule(schedule_id)
    if not schedule:
        raise HTTPException(status_code=404, detail="Schedule not found")
    return schedule

# UPDATE
@router.put("/{schedule_id}", response_model=ScheduleRead)
async def update_schedule(
    schedule_id: str, 
    schedule: ScheduleUpdate,
    user_id: str = Depends(get_current_user_id)):
    updated = await schedule_crud.update_schedule(schedule_id, schedule, user_id=user_id)
    if not updated:
        raise HTTPException(status_code=404, detail="Schedule not found")
    return updated

# DELETE
@router.delete("/{schedule_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_schedule(schedule_id: str):
    deleted = await schedule_crud.delete_schedule(schedule_id)
    if not deleted:
        raise HTTPException(status_code=404, detail="Schedule not found")
    return None
