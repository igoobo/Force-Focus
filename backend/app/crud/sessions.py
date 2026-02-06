# backend/app/crud/sessions.py

from datetime import datetime, timezone
from typing import List, Optional

from bson import ObjectId
from bson.errors import InvalidId
from fastapi import HTTPException

from app.db.mongo import get_db
from app.schemas.session import SessionCreate, SessionUpdate, SessionRead

from app.crud.events import get_events  # 이벤트 조회를 위해 추가

async def get_session_context_for_llm(user_id: str, session_id: str) -> str:
    """
    특정 세션의 이벤트들을 LLM이 분석하기 좋은 텍스트로 변환합니다.
    """
    events = await get_events(user_id=user_id, session_id=session_id, limit=300)
    if not events:
        return "기록된 이벤트가 없습니다."

    # 시간순 정렬 및 요약
    events.reverse()
    event_summary = []
    for e in events:
        time = e.timestamp.strftime("%H:%M")
        event_summary.append(f"[{time}] 앱: {e.app_name}, 창 제목: {e.window_title}")

    return "\n".join(event_summary)


def get_sessions_collection():
    """
    Motor DB 핸들에서 sessions 컬렉션을 가져옵니다.
    """
    return get_db()["sessions"]


def serialize_session(session) -> SessionRead:
    """
    Mongo document(dict) -> SessionRead
    """
    return SessionRead(
        id=str(session["_id"]),
        user_id=session["user_id"],
        task_id=session.get("task_id"),
        profile_id=session.get("profile_id"),
        start_time=session["start_time"],
        end_time=session.get("end_time"),
        duration=session.get("duration"),
        status=session.get("status", "active"),
        goal_duration=session.get("goal_duration"),
        interruption_count=session.get("interruption_count", 0),
    )


def _safe_object_id(session_id: str) -> ObjectId:
    try:
        return ObjectId(session_id)
    except (InvalidId, TypeError):
        raise HTTPException(status_code=400, detail="Invalid session_id")


def _utcnow() -> datetime:
    return datetime.now(timezone.utc)


def _ensure_aware_utc(dt: datetime) -> datetime:
    """
    start_time/end_time에 naive가 들어오는 케이스 방어.
    naive면 UTC로 간주해서 tzinfo를 붙임.
    """
    if dt is None:
        return dt
    if dt.tzinfo is None:
        return dt.replace(tzinfo=timezone.utc)
    return dt.astimezone(timezone.utc)


def _compute_duration_seconds(start_time: datetime, end_time: datetime) -> float:
    """
    duration을 초 단위로 계산. 음수 방지.
    """
    st = _ensure_aware_utc(start_time)
    et = _ensure_aware_utc(end_time)

    sec = (et - st).total_seconds()
    if sec < 0:
        raise HTTPException(status_code=400, detail="end_time must be after start_time")
    return float(sec)


def _strip_or_none(v: Optional[str]) -> Optional[str]:
    """
    CRUD 안전망: Optional[str]가 DB로 들어가기 전 한번 더 정리
    """
    if v is None:
        return None
    if not isinstance(v, str):
        return v
    s = v.strip()
    return s or None


async def _cancel_existing_active_sessions(user_id: str) -> None:
    """
    정책: 유저당 active 세션은 1개만 허용.
    start_session 호출 시 기존 active 세션을 자동 cancelled 처리.
    - end_time=now
    - duration 계산해서 저장
    """
    col = get_sessions_collection()
    now = _utcnow()

    cursor = col.find({"user_id": user_id, "status": "active"}).sort("start_time", -1)
    active_sessions = await cursor.to_list(length=100)  # 충분히 크게

    for s in active_sessions:
        # 이미 end_time이 있다면 굳이 건드리지 않음(데이터 꼬임 방지)
        if s.get("end_time") is not None:
            continue

        try:
            duration = _compute_duration_seconds(s["start_time"], now)
        except HTTPException:
            # start_time이 이상하면 duration 저장 없이라도 종료 처리
            duration = None

        update_doc = {
            "status": "cancelled",
            "end_time": now,
        }
        if duration is not None:
            update_doc["duration"] = duration

        await col.update_one({"_id": s["_id"]}, {"$set": update_doc})


# CREATE (START)
async def start_session(user_id: str, data: SessionCreate) -> SessionRead:
    col = get_sessions_collection()

    # 1) 기존 active 자동 종료(정책 적용)
    await _cancel_existing_active_sessions(user_id)

    # 2) start_time 보정
    start_time = data.start_time or _utcnow()
    start_time = _ensure_aware_utc(start_time)

    # 3) id 계열 안전망 strip (스키마에서 처리되지만 DB 보호용)
    task_id = _strip_or_none(data.task_id)
    profile_id = _strip_or_none(data.profile_id)

    doc = {
        "user_id": user_id,
        "task_id": task_id,
        "profile_id": profile_id,
        "start_time": start_time,
        "end_time": None,
        "duration": None,
        "status": "active",
        "goal_duration": data.goal_duration,
        "interruption_count": 0,
    }

    result = await col.insert_one(doc)
    created = await col.find_one({"_id": result.inserted_id})
    if not created:
        raise HTTPException(status_code=500, detail="Failed to create session")
    return serialize_session(created)


# READ ALL (user 기준)
async def get_sessions(
    user_id: str,
    status: Optional[str] = None,
    limit: int = 50
) -> List[SessionRead]:
    col = get_sessions_collection()

    query = {"user_id": user_id}
    if status:
        query["status"] = status

    cursor = col.find(query).sort("start_time", -1).limit(limit)
    sessions = await cursor.to_list(length=limit)
    return [serialize_session(s) for s in sessions]


# READ ONE
async def get_session(session_id: str) -> Optional[SessionRead]:
    col = get_sessions_collection()
    oid = _safe_object_id(session_id)

    session = await col.find_one({"_id": oid})
    if not session:
        return None
    return serialize_session(session)


# READ CURRENT (active 세션 1개)
async def get_current_session(user_id: str) -> Optional[SessionRead]:
    col = get_sessions_collection()

    session = await col.find_one(
        {"user_id": user_id, "status": "active"},
        sort=[("start_time", -1)],
    )
    if not session:
        return None
    return serialize_session(session)


# UPDATE (END 포함)
async def update_session(user_id: str, session_id: str, data: SessionUpdate) -> SessionRead:
    col = get_sessions_collection()
    oid = _safe_object_id(session_id)

    existing = await col.find_one({"_id": oid})
    if not existing:
        raise HTTPException(status_code=404, detail="Session not found")

    # 다른 유저 세션 수정 방지
    if existing.get("user_id") != user_id:
        raise HTTPException(status_code=403, detail="Forbidden")

    update_doc = {}

    if data.end_time is not None:
        end_time = _ensure_aware_utc(data.end_time)
        update_doc["end_time"] = end_time
        update_doc["duration"] = _compute_duration_seconds(existing["start_time"], end_time)

    if data.status is not None:
        # 스키마에서 strip/blank 방지하지만 안전망
        update_doc["status"] = _strip_or_none(data.status) or data.status

    if data.goal_duration is not None:
        update_doc["goal_duration"] = data.goal_duration

    if data.interruption_count is not None:
        if data.interruption_count < 0:
            raise HTTPException(status_code=400, detail="interruption_count must be >= 0")
        update_doc["interruption_count"] = data.interruption_count

    if not update_doc:
        return serialize_session(existing)

    await col.update_one({"_id": oid}, {"$set": update_doc})
    updated = await col.find_one({"_id": oid})
    if not updated:
        raise HTTPException(status_code=500, detail="Failed to update session")
    return serialize_session(updated)

# END 세션 (편의 함수)
async def end_session(
    user_id: str,
    session_id: str,
    end_time: datetime,
    status: str = "completed",
) -> SessionRead:
    return await update_session(
        user_id,
        session_id,
        SessionUpdate(end_time=end_time, status=status),
    )
