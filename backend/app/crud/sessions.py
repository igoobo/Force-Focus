# backend/app/crud/sessions.py

from datetime import datetime, timezone
from typing import List, Optional, Literal

from bson import ObjectId
from bson.errors import InvalidId
from fastapi import HTTPException

from app.db.mongo import get_db
from app.schemas.session import SessionCreate, SessionUpdate, SessionRead

from app.crud.events import get_events


SessionStatus = Literal["active", "completed", "cancelled"]


async def get_session_full_context(user_id: str, session_id: str) -> str:
    """
    세션 정보와 이벤트 목록을 바탕으로 피드백을 구성합니다.
    [개선] task_id를 통해 작업명과 허용 프로그램 정보를 AI 컨텍스트에 추가합니다.
    """
    col = get_sessions_collection()
    db = get_db()
    uid = str(user_id)

    existing = await _find_session_doc_for_user(uid, session_id)

    if not existing:
        return "죄송합니다. 분석할 세션 기록을 찾을 수 없습니다."

    task_id = existing.get("task_id")
    task_context = "기본 세션 (사용자가 특정 작업을 설정하지 않았습니다.)"
    if task_id:
        try:
            task_data = await db["tasks"].find_one({"_id": _safe_object_id(task_id)})
            if task_data:
                t_name = task_data.get("name", "알 수 없음")
                t_apps = task_data.get("target_executable", "없음")
                task_context = f"작업명: {t_name} | 허용 프로그램: {t_apps}"
        except Exception:
            task_context = "작업 정보를 불러오는 중 오류가 발생했습니다."
    else:
        task_context += " AI는 활동 로그와 창 제목을 분석하여 사용자가 어떤 성격의 업무(예: 코딩, 기획, 단순 웹서핑 등)를 수행 중이었는지 스스로 판단하세요."

    lookup_id = existing.get("client_session_id") or session_id

    start_time = existing["start_time"]
    start_str = start_time.strftime("%Y년 %m월 %d일 %H시 %M분")
    duration = existing.get("duration")

    if duration:
        mins, secs = divmod(int(duration), 60)
        duration_val = f"{mins}분 {secs}초"
    else:
        duration_val = "진행 중인 세션"

    events = await get_events(user_id=uid, session_id=lookup_id, limit=500)
    app_stats_context = ""

    if events:
        app_counts = {}
        for e in events:
            name = e.app_name or "알 수 없음"
            app_counts[name] = app_counts.get(name, 0) + 1

        sorted_apps = sorted(app_counts.items(), key=lambda x: x[1], reverse=True)
        top_app, top_count = sorted_apps[0]
        distraction_ratio = (top_count / len(events)) * 100

        app_stats_context = "### 📊 데이터 기반 활동 분석\n"
        app_stats_context += f"- 가장 높은 비중의 앱: {top_app}\n"
        app_stats_context += f"- 해당 앱 점유율: {distraction_ratio:.1f}%\n\n"

    context = "## 🎯 이번 세션 분석 리포트\n\n"
    context += f"**세션 시작:** {start_str}\n"
    context += f"**총 집중 시간:** {duration_val}\n"
    context += f"**설정된 작업 목표:** {task_context}\n\n"

    context += app_stats_context

    context += "### 🔍 활동 타임라인 상세\n"
    if not events:
        context += "- 수집된 상세 활동 로그가 없습니다.\n"
    else:
        events.reverse()
        for i, e in enumerate(events, 1):
            ts = e.timestamp.strftime("%H:%M:%S")
            activity = f"[{ts}] {e.app_name} - {e.window_title}" if e.app_name else f"[{ts}] {e.window_title}"
            context += f"{i}. {activity}\n"

    context += "\n---\n"
    context += "### 💡 코치 시스템 지침:\n"
    context += "1. '설정된 작업 목표'의 허용 프로그램과 실제 '활동 타임라인'을 대조하여 집중도를 평가하세요.\n"
    context += "2. 제공된 '데이터 기반 활동 분석'의 앱 이름과 점유율을 응답 필드에 정확히 반영하세요.\n"
    context += "3. 각 피드백 항목은 항목당 최소 2~3문장 이상의 상세한 설명으로 작성하세요.\n"
    context += "4. 회복 전략(recovery_strategies)은 서로 다른 카테고리로 2개를 작성하고 전체 500자 이상을 유지하세요.\n"

    return context


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
        client_session_id=session.get("client_session_id"),
        task_id=session.get("task_id"),
        profile_id=session.get("profile_id"),
        start_time=session["start_time"],
        end_time=session.get("end_time"),
        duration=session.get("duration"),
        status=session.get("status", "active"),
        goal_duration=session.get("goal_duration"),
        interruption_count=session.get("interruption_count", 0),
    )


def _safe_object_id(session_id: str):
    try:
        return ObjectId(session_id)
    except (InvalidId, TypeError):
        return session_id


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
    if v is None:
        return None
    if not isinstance(v, str):
        return v
    s = v.strip()
    return s or None


async def _find_session_doc_for_user(user_id: str, session_id: str):
    col = get_sessions_collection()
    oid = _safe_object_id(session_id)
    return await col.find_one(
        {
            "$or": [{"_id": oid}, {"client_session_id": session_id}],
            "user_id": str(user_id),
        }
    )


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
    active_sessions = await cursor.to_list(length=100)

    for s in active_sessions:
        if s.get("end_time") is not None:
            continue

        try:
            duration = _compute_duration_seconds(s["start_time"], now)
        except HTTPException:
            duration = None

        update_doc = {
            "status": "cancelled",
            "end_time": now,
        }
        if duration is not None:
            update_doc["duration"] = duration

        await col.update_one({"_id": s["_id"]}, {"$set": update_doc})


async def start_session(user_id: str, data: SessionCreate) -> SessionRead:
    col = get_sessions_collection()

    await _cancel_existing_active_sessions(user_id)

    start_time = data.start_time if data.start_time else _utcnow()
    start_time = _ensure_aware_utc(start_time)

    task_id = _strip_or_none(data.task_id)
    profile_id = _strip_or_none(data.profile_id)
    client_sid = _strip_or_none(data.client_session_id)

    doc = {
        "user_id": user_id,
        "client_session_id": client_sid,
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


async def get_sessions(
    user_id: str,
    status: Optional[SessionStatus] = None,
    limit: int = 50,
) -> List[SessionRead]:
    col = get_sessions_collection()

    query = {"user_id": user_id}
    if status:
        query["status"] = status

    cursor = col.find(query).sort("start_time", -1).limit(limit)
    sessions = await cursor.to_list(length=limit)
    return [serialize_session(s) for s in sessions]


async def get_session(user_id: str, session_id: str) -> Optional[SessionRead]:
    session = await _find_session_doc_for_user(user_id, session_id)
    if not session:
        return None
    return serialize_session(session)


async def get_current_session(user_id: str) -> Optional[SessionRead]:
    col = get_sessions_collection()

    session = await col.find_one(
        {"user_id": user_id, "status": "active"},
        sort=[("start_time", -1)],
    )
    if not session:
        return None
    return serialize_session(session)


async def update_session(user_id: str, session_id: str, data: SessionUpdate) -> SessionRead:
    col = get_sessions_collection()
    uid = str(user_id)

    existing = await _find_session_doc_for_user(uid, session_id)
    if not existing:
        raise HTTPException(status_code=404, detail="Session not found")

    target_id = existing["_id"]
    update_doc = {}

    if not existing.get("client_session_id") and "local-" in str(session_id):
        update_doc["client_session_id"] = session_id

    actual_end_time = data.end_time
    if actual_end_time is None and data.end_time_s is not None:
        actual_end_time = datetime.fromtimestamp(data.end_time_s, tz=timezone.utc)

    if actual_end_time is not None:
        end_time = _ensure_aware_utc(actual_end_time)
        update_doc["end_time"] = end_time
        update_doc["duration"] = _compute_duration_seconds(existing["start_time"], end_time)

    if data.status is not None:
        update_doc["status"] = data.status
    elif actual_end_time is not None:
        update_doc["status"] = "completed"

    if data.goal_duration is not None:
        update_doc["goal_duration"] = data.goal_duration

    if data.interruption_count is not None:
        update_doc["interruption_count"] = data.interruption_count

    if not update_doc:
        return serialize_session(existing)

    await col.update_one({"_id": target_id}, {"$set": update_doc})
    updated = await col.find_one({"_id": target_id})
    if not updated:
        raise HTTPException(status_code=500, detail="Failed to update session")
    return serialize_session(updated)


async def end_session(
    user_id: str,
    session_id: str,
    end_time: datetime,
    status: SessionStatus = "completed",
) -> SessionRead:
    return await update_session(
        user_id,
        session_id,
        SessionUpdate(end_time=end_time, status=status),
    )