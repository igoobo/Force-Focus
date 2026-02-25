# backend/app/api/endpoints/user/me.py

from fastapi import APIRouter, Depends, HTTPException, status

from app.api.deps import get_current_user_id
from app.schemas.user import (
    UserRead,
    SettingsPatch,
    SuccessMessage,
    FCMTokenAdd,
    FCMTokenDelete,
    BlockedAppAdd,
    BlockedAppDelete,
)
from app.crud import users as users_crud

router = APIRouter(prefix="/users", tags=["Users"])


def _to_user_read(user) -> UserRead:
    """
    UserInDB -> UserRead 변환 시 id(ObjectId)를 str로 강제 변환.
    """
    data = user.model_dump(by_alias=False)
    data["id"] = str(user.id)
    return UserRead(**data)


def _normalize_settings(incoming: dict) -> dict:
    """
    settings payload를 DB에 저장하기 전에 정규화.
    - legacy 키를 canonical 키로 변환
    - 불필요한 키 정리 (선택)
    """
    s = dict(incoming or {})

    # ✅ key 공백/빈키 제거 (프리폼 dict 방어)
    cleaned = {}
    for k, v in s.items():
        if not isinstance(k, str):
            continue
        kk = k.strip()
        if kk == "":
            continue
        cleaned[kk] = v
    s = cleaned

    # --- legacy key normalize ---
    # daily_goal_min -> daily_goal_minutes
    if "daily_goal_min" in s:
        # canonical이 없으면 이 값을 승격
        if "daily_goal_minutes" not in s:
            s["daily_goal_minutes"] = s["daily_goal_min"]
        # legacy 키는 제거(중복 방지)
        s.pop("daily_goal_min", None)

    # --- (옵션) 허용 키만 유지하고 싶으면 아래를 활성화 ---
    # allowed = {"timezone", "focus_mode", "daily_goal_minutes"}
    # s = {k: v for k, v in s.items() if k in allowed}

    return s


@router.get("/me", response_model=UserRead)
async def read_my_profile(
    user_id: str = Depends(get_current_user_id),
):
    user = await users_crud.get_user_by_id(user_id)
    if user is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="User not found")

    return _to_user_read(user)


@router.patch("/me/settings", response_model=UserRead)
async def update_my_settings(
    payload: SettingsPatch,
    user_id: str = Depends(get_current_user_id),
):
    normalized = _normalize_settings(payload.settings)

    user = await users_crud.update_settings(user_id=user_id, settings=normalized)
    if user is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Failed to update settings")

    return _to_user_read(user)


@router.post("/me/fcm-tokens", response_model=UserRead)
async def add_my_fcm_token(
    payload: FCMTokenAdd,
    user_id: str = Depends(get_current_user_id),
):
    user = await users_crud.add_fcm_token(user_id=user_id, token=payload.token)
    if user is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Failed to add FCM token")

    return _to_user_read(user)


@router.delete("/me/fcm-tokens", response_model=SuccessMessage)
async def delete_my_fcm_token(
    payload: FCMTokenDelete,
    user_id: str = Depends(get_current_user_id),
):
    # 스키마에 resolved_token()이 있으니 그걸 쓰는 게 제일 안전
    token = payload.resolved_token()

    user = await users_crud.remove_fcm_token(user_id=user_id, token=token)
    if user is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Failed to remove FCM token")

    return SuccessMessage(message="FCM token removed")


@router.post("/me/blocked-apps", response_model=UserRead)
async def add_my_blocked_app(
    payload: BlockedAppAdd,
    user_id: str = Depends(get_current_user_id),
):
    user = await users_crud.add_blocked_app(user_id=user_id, app_name=payload.app_name)
    if user is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Failed to add blocked app")

    return _to_user_read(user)


@router.delete("/me/blocked-apps", response_model=UserRead)
async def delete_my_blocked_app(
    payload: BlockedAppDelete,
    user_id: str = Depends(get_current_user_id),
):
    user = await users_crud.remove_blocked_app(user_id=user_id, app_name=payload.app_name)
    if user is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Failed to remove blocked app")

    return _to_user_read(user)
