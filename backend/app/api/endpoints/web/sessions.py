# backend/app/api/endpoints/web/sessions.py

from fastapi import APIRouter, Depends, HTTPException, status
from typing import List, Optional, Literal

from app.api.deps import get_current_user_id
from app.schemas.session import SessionCreate, SessionUpdate, SessionRead
from app.crud import sessions as session_crud

router = APIRouter(prefix="/sessions", tags=["Sessions"])


@router.post("/start", response_model=SessionRead, status_code=status.HTTP_201_CREATED)
async def start_session(
    payload: SessionCreate,
    user_id: str = Depends(get_current_user_id),
):
    return await session_crud.start_session(user_id, payload)


@router.get("/", response_model=List[SessionRead])
async def read_sessions(
    status: Optional[Literal["active", "completed", "cancelled"]] = None,
    limit: int = 50,
    user_id: str = Depends(get_current_user_id),
):
    return await session_crud.get_sessions(user_id, status=status, limit=limit)


@router.get("/current", response_model=SessionRead)
async def read_current_session(
    user_id: str = Depends(get_current_user_id),
):
    session = await session_crud.get_current_session(user_id)
    if not session:
        raise HTTPException(status_code=404, detail="No active session")
    return session


@router.get("/{session_id}", response_model=SessionRead)
async def read_session(
    session_id: str,
    user_id: str = Depends(get_current_user_id),
):
    session = await session_crud.get_session(user_id, session_id)
    if not session:
        raise HTTPException(status_code=404, detail="Session not found")
    return session


@router.put("/{session_id}", response_model=SessionRead)
async def update_session(
    session_id: str,
    payload: SessionUpdate,
    user_id: str = Depends(get_current_user_id),
):
    return await session_crud.update_session(user_id, session_id, payload)