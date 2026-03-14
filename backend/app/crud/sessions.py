# backend/app/crud/sessions.py

from datetime import datetime, timezone
from typing import List, Optional

from bson import ObjectId
from bson.errors import InvalidId
from fastapi import HTTPException

from app.db.mongo import get_db
from app.schemas.session import SessionCreate, SessionUpdate, SessionRead

from app.crud.events import get_events  # 이벤트 조회를 위해 추가

async def get_session_full_context(user_id: str, session_id: str) -> str:
    """
    세션 정보와 이벤트 목록을 바탕으로 피드백을 구성합니다.
    [개선] task_id를 통해 작업명과 허용 프로그램 정보를 AI 컨텍스트에 추가합니다.
    """
    col = get_sessions_collection()
    db = get_db() # Task 조회를 위한 DB 핸들
    uid = str(user_id)
    oid = _safe_object_id(session_id)

    # 1. 세션 본체 찾기
    existing = await col.find_one({
        "$or": [{"_id": oid}, {"client_session_id": session_id}],
        "user_id": uid
    })
    
    if not existing:
        existing = await col.find_one(
            {"user_id": uid},
            sort=[("end_time", -1), ("start_time", -1)]
        )

    if not existing:
        return "죄송합니다. 분석할 세션 기록을 찾을 수 없습니다."

    # 작업 상세 정보 조회 로직
    task_id = existing.get("task_id")
    task_context = "기본 세션 (사용자가 특정 작업을 설정하지 않았습니다.)"
    if task_id:
        try:
            task_data = await db["tasks"].find_one({"_id": _safe_object_id(task_id)})
            if task_data:
                # 이미지 데이터 구조 참조: name, target_executable
                t_name = task_data.get("name", "알 수 없음")
                t_apps = task_data.get("target_executable", "없음")
                task_context = f"작업명: {t_name} | 허용 프로그램: {t_apps}"
        except Exception:
            task_context = "작업 정보를 불러오는 중 오류가 발생했습니다."
    else:
        # 작업이 없을 때 AI에게 부여하는 추가 컨텍스트
        task_context += " AI는 활동 로그와 창 제목을 분석하여 사용자가 어떤 성격의 업무(예: 코딩, 기획, 단순 웹서핑 등)를 수행 중이었는지 스스로 판단하세요."

    lookup_id = existing.get("client_session_id") or session_id

    # 사용자 친화적인 세션 정보 요약
    start_time = existing["start_time"]
    start_str = start_time.strftime("%Y년 %m월 %d일 %H시 %M분")
    duration = existing.get("duration")
    
    if duration:
        mins, secs = divmod(int(duration), 60)
        duration_val = f"{mins}분 {secs}초"
    else:
        duration_val = "진행 중인 세션"

    # 방해 요소 및 앱 사용 통계 계산 로직 유지
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
        
        app_stats_context = f"### 📊 데이터 기반 활동 분석\n"
        app_stats_context += f"- 가장 높은 비중의 앱: {top_app}\n"
        app_stats_context += f"- 해당 앱 점유율: {distraction_ratio:.1f}%\n\n"

    # LLM 전달 프롬프트 구성 (task_context 주입)
    context = f"## 🎯 이번 세션 분석 리포트\n\n"
    context += f"**세션 시작:** {start_str}\n"
    context += f"**총 집중 시간:** {duration_val}\n"
    context += f"**설정된 작업 목표:** {task_context}\n\n" # 추가됨
    
    context += app_stats_context
    
    context += "### 🔍 활동 타임라인 상세\n"
    if not events:
        context += "- 수집된 상세 활동 로그가 없습니다.\n"
    else:
        events.reverse()
        for i, e in enumerate(events, 1):
            time = e.timestamp.strftime("%H:%M:%S")
            activity = f"[{time}] {e.app_name} - {e.window_title}" if e.app_name else f"[{time}] {e.window_title}"
            context += f"{i}. {activity}\n"

    # 코치 시스템 지침 유지 및 보강
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
    start_time = data.start_time if data.start_time else _utcnow()
    start_time = _ensure_aware_utc(start_time)

    # 3) id 계열 안전망 strip (스키마에서 처리되지만 DB 보호용)
    task_id = _strip_or_none(data.task_id)
    profile_id = _strip_or_none(data.profile_id)

    # [수정] 스키마 필드명인 client_session_id에 직접 접근하여 안전하게 수신
    client_sid = data.client_session_id 

    doc = {
        "user_id": user_id,
        "client_session_id": client_sid, # 매핑 필드 추가
        "task_id": task_id,
        "profile_id": profile_id,
        "start_time": start_time,
        "end_time": None,
        "duration": None,
        "status": "active",
        "goal_duration": data.goal_duration if data.goal_duration else 0,
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

    # [수정] client_session_id로도 조회 가능하도록 필터 확장
    session = await col.find_one({
        "$or": [{"_id": oid}, {"client_session_id": session_id}]
    })
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
    uid = str(user_id)
    oid = _safe_object_id(session_id)

    # 1) 기본 ID 기반 조회 시도 (매핑 포함)
    existing = await col.find_one({
        "$or": [{"_id": oid}, {"client_session_id": session_id}],
        "user_id": uid
    })

    # 2) [핵심] ID로 못 찾았을 경우, 해당 유저의 현재 활성 세션을 검색
    if not existing:
        existing = await col.find_one(
            {"user_id": uid, "status": "active"},
            sort=[("start_time", -1)]
        )

    if not existing:
        raise HTTPException(status_code=404, detail="Session not found")

    target_id = existing["_id"]
    update_doc = {}

    # [수정] 업데이트 중 앱의 ID가 확인되면 client_session_id 매핑 정보 보강
    if not existing.get("client_session_id") and "local-" in str(session_id):
        update_doc["client_session_id"] = session_id

    # 데스크탑 앱 호환: 앱에서 명시적 종료 시간을 주지 않더라도 서버 시간 사용
    actual_end_time = data.end_time
    if actual_end_time is None:
        ts = getattr(data, "end_time_s", None)
        actual_end_time = datetime.fromtimestamp(ts, tz=timezone.utc) if ts else _utcnow()

    if actual_end_time is not None:
        end_time = _ensure_aware_utc(actual_end_time)
        update_doc["end_time"] = end_time
        update_doc["duration"] = _compute_duration_seconds(existing["start_time"], end_time)

    # 상태 업데이트 및 기본값 설정
    update_doc["status"] = _strip_or_none(data.status) or "completed"

    if data.goal_duration is not None:
        update_doc["goal_duration"] = data.goal_duration

    if data.interruption_count is not None:
        if data.interruption_count < 0:
            raise HTTPException(status_code=400, detail="interruption_count must be >= 0")
        update_doc["interruption_count"] = data.interruption_count

    await col.update_one({"_id": target_id}, {"$set": update_doc})
    updated = await col.find_one({"_id": target_id})
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