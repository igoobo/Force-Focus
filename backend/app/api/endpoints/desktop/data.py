from fastapi import APIRouter, Depends, HTTPException
from typing import List

from app.crud import tasks as task_crud
from app.crud import schedules as schedule_crud

from app.schemas.task import TaskRead
from app.schemas.schedule import ScheduleRead

# [데스크톱 전용] JWT 인증 의존성 (웹 세션 대신 토큰 사용)
from app.api.deps import get_current_user_id

router = APIRouter()

# --------------------------------------------------------------------------
# GET /api/v1/desktop/data/tasks
# 설명: 로그인한 사용자의 모든 할 일(Task) 목록을 조회
# --------------------------------------------------------------------------
@router.get("/tasks", response_model=List[TaskRead])
async def read_my_tasks(
    # 헤더의 JWT 토큰을 검증하고 user_id를 추출
    user_id: str = Depends(get_current_user_id)
):
    # CRUD 모듈을 재사용하여 DB 조회
    tasks = await task_crud.get_tasks(user_id)
    return tasks

# --------------------------------------------------------------------------
# GET /api/v1/desktop/data/schedules
# 설명: 로그인한 사용자의 모든 스케줄 목록을 조회
# --------------------------------------------------------------------------
@router.get("/schedules", response_model=List[ScheduleRead])
async def read_my_schedules(
    user_id: str = Depends(get_current_user_id)
):
    # CRUD 모듈 재사용
    schedules = await schedule_crud.get_schedules(user_id)
    return schedules