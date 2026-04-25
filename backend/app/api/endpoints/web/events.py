# backend/app/api/endpoints/web/events.py

from fastapi import APIRouter, Depends, HTTPException, status, Query
from typing import List, Optional
from datetime import datetime

from app.api.deps import get_current_user_id
from app.schemas.event import EventCreate, EventCreateResponse, EventRead
from app.crud import events as event_crud

router = APIRouter(prefix="/events", tags=["Events"])


@router.post("/", response_model=EventCreateResponse, status_code=status.HTTP_201_CREATED)
async def create_event(
    payload: EventCreate,
    user_id: str = Depends(get_current_user_id),
):
    event_id = await event_crud.create_event_for_user(user_id, payload)
    return EventCreateResponse(event_id=event_id)


@router.get("/", response_model=List[EventRead])
async def read_events(
    session_id: Optional[str] = Query(None),
    start_time: Optional[datetime] = Query(None, description="ISO8601 datetime"),
    end_time: Optional[datetime] = Query(None, description="ISO8601 datetime"),
    limit: int = Query(100, ge=1, le=1000),
    user_id: str = Depends(get_current_user_id),
):
    return await event_crud.get_events(
        user_id=user_id,
        session_id=session_id,
        start_time=start_time,
        end_time=end_time,
        limit=limit,
    )


@router.get("/{event_id}", response_model=EventRead)
async def read_event(
    event_id: str,
    user_id: str = Depends(get_current_user_id),
):
    event = await event_crud.get_event(user_id, event_id)
    if not event:
        raise HTTPException(status_code=404, detail="Event not found")
    return event