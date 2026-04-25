# backend/app/crud/users.py

from datetime import datetime, timezone
from typing import Optional, Dict, Any, Union

from bson import ObjectId
from bson.errors import InvalidId

from app.db.mongo import get_db
from app.models.user import UserInDB


def get_users_collection():
    """
    Motor DB 핸들에서 users 컬렉션을 가져옵니다.
    connect_to_mongo() 이후에 db가 세팅되어 있어야 합니다.
    """
    return get_db()["users"]


def _strip_or_none(v):
    if v is None:
        return None
    if not isinstance(v, str):
        return v
    s = v.strip()
    return s or None


def _safe_object_id(user_id: Union[str, ObjectId]) -> Optional[ObjectId]:
    """
    str/ObjectId 입력을 안전하게 ObjectId로 변환합니다.
    """
    if isinstance(user_id, ObjectId):
        return user_id

    if isinstance(user_id, str):
        user_id = user_id.strip()

    try:
        return ObjectId(user_id)
    except (InvalidId, TypeError):
        return None


def _id_filter(user_id: Union[str, ObjectId]) -> Dict[str, Any]:
    """
    users 컬렉션의 _id 타입이 ObjectId / string 혼재된 상황을 모두 커버하는 필터.
    현재는 과거 데이터 호환을 위해 유지하되,
    외부 계약상 user id는 ObjectId hex 문자열을 기준으로 사용한다.
    """
    if isinstance(user_id, str):
        user_id = user_id.strip()

    oid = _safe_object_id(user_id)

    if isinstance(user_id, str) and oid is not None:
        return {"$or": [{"_id": oid}, {"_id": user_id}]}

    if oid is not None:
        return {"_id": oid}

    return {"_id": user_id}


def _now() -> datetime:
    return datetime.now(timezone.utc)


async def get_user_by_id(user_id: Union[str, ObjectId]) -> Optional[UserInDB]:
    user = await get_users_collection().find_one(_id_filter(user_id))
    return UserInDB(**user) if user else None


async def get_user_by_google_id(google_id: str) -> Optional[UserInDB]:
    google_id = _strip_or_none(google_id) or google_id

    user = await get_users_collection().find_one({"google_id": google_id})
    return UserInDB(**user) if user else None


async def create_user(
    *,
    email: str,
    google_id: str,
    user_settings: Optional[Dict[str, Any]] = None
) -> UserInDB:
    now = _now()

    email = _strip_or_none(email) or email
    google_id = _strip_or_none(google_id) or google_id

    user_data = {
        "email": email,
        "google_id": google_id,
        "created_at": now,
        "last_login_at": now,
        "settings": user_settings or {},
        "fcm_tokens": [],
        "blocked_apps": [],
    }

    result = await get_users_collection().insert_one(user_data)
    user_data["_id"] = result.inserted_id
    return UserInDB(**user_data)


async def update_last_login(user_id: Union[str, ObjectId]) -> Optional[UserInDB]:
    result = await get_users_collection().update_one(
        _id_filter(user_id),
        {"$set": {"last_login_at": _now()}},
    )
    if result.matched_count == 0:
        return None

    return await get_user_by_id(user_id)


async def update_settings(
    user_id: Union[str, ObjectId],
    settings: Dict[str, Any],
) -> Optional[UserInDB]:
    update_doc: Dict[str, Any] = {
        "$unset": {"settings.daily_goal_min": ""}
    }

    if settings:
        safe_items = {}
        for k, v in settings.items():
            if not isinstance(k, str):
                continue

            kk = k.strip()
            if kk == "":
                continue

            safe_items[kk] = v

        if safe_items:
            update_doc["$set"] = {f"settings.{k}": v for k, v in safe_items.items()}

    result = await get_users_collection().update_one(
        _id_filter(user_id),
        update_doc,
    )
    if result.matched_count == 0:
        return None

    return await get_user_by_id(user_id)


async def add_fcm_token(user_id: Union[str, ObjectId], token: str) -> Optional[UserInDB]:
    token = _strip_or_none(token) or token

    result = await get_users_collection().update_one(
        _id_filter(user_id),
        {"$addToSet": {"fcm_tokens": token}},
    )
    if result.matched_count == 0:
        return None

    return await get_user_by_id(user_id)


async def remove_fcm_token(
    user_id: Union[str, ObjectId],
    token: str,
) -> Optional[UserInDB]:
    token = _strip_or_none(token)
    if not token:
        return None

    result = await get_users_collection().update_one(
        _id_filter(user_id),
        {"$pull": {"fcm_tokens": token}},
    )
    if result.matched_count == 0:
        return None

    return await get_user_by_id(user_id)


async def add_blocked_app(user_id: Union[str, ObjectId], app_name: str) -> Optional[UserInDB]:
    app_name = _strip_or_none(app_name) or app_name

    result = await get_users_collection().update_one(
        _id_filter(user_id),
        {"$addToSet": {"blocked_apps": app_name}},
    )
    if result.matched_count == 0:
        return None

    return await get_user_by_id(user_id)


async def remove_blocked_app(user_id: Union[str, ObjectId], app_name: str) -> Optional[UserInDB]:
    app_name = _strip_or_none(app_name) or app_name

    result = await get_users_collection().update_one(
        _id_filter(user_id),
        {"$pull": {"blocked_apps": app_name}},
    )
    if result.matched_count == 0:
        return None

    return await get_user_by_id(user_id)


async def delete_user(user_id: Union[str, ObjectId]) -> bool:
    """
    ID를 기반으로 사용자를 컬렉션에서 영구 삭제합니다.
    삭제 계열 함수이므로 성공 여부(bool)를 반환합니다.
    """
    user_filter = _id_filter(user_id)
    result = await get_users_collection().delete_one(user_filter)
    return result.deleted_count > 0