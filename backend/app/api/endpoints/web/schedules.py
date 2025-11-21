# backend/app/api/endpoints/schedules.py
from fastapi import APIRouter, HTTPException, Depends, status
from typing import List

from app.schemas.schedule import ScheduleCreate, ScheduleUpdate, ScheduleRead
from app.crud import schedules as schedule_crud

router = APIRouter(tags=["Schedules"])

# TODO: JWT 인증 붙이면 교체 예정
USER_ID = "test_user_123"

# CREATE
@router.post("/", response_model=ScheduleRead, status_code=status.HTTP_201_CREATED)
async def create_schedule(schedule: ScheduleCreate):
    return await schedule_crud.create_schedule(USER_ID, schedule)

# READ ALL
@router.get("/", response_model=List[ScheduleRead])
async def read_schedules():
    return await schedule_crud.get_schedules(USER_ID)

# READ ONE
@router.get("/{schedule_id}", response_model=ScheduleRead)
async def read_schedule(schedule_id: str):
    schedule = await schedule_crud.get_schedule(schedule_id)
    if not schedule:
        raise HTTPException(status_code=404, detail="Schedule not found")
    return schedule

# UPDATE
@router.put("/{schedule_id}", response_model=ScheduleRead)
async def update_schedule(schedule_id: str, schedule: ScheduleUpdate):
    updated = await schedule_crud.update_schedule(schedule_id, schedule)
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
