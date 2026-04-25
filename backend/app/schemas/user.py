# backend/app/schemas/user.py

from datetime import datetime
from typing import Optional, List, Dict, Any, Literal

from pydantic import BaseModel, EmailStr, Field, ConfigDict, field_validator


def _strip_and_reject_blank(v: str, field_name: str) -> str:
    """
    문자열 양쪽 공백 제거 후,
    빈 문자열이면 ValidationError 유발을 위해 ValueError 발생.
    """
    if v is None:
        return v
    if not isinstance(v, str):
        return v

    stripped = v.strip()
    if stripped == "":
        raise ValueError(f"{field_name} must not be blank")
    return stripped


def _strip_to_none(v):
    """
    Optional[str] 입력에서:
    - None은 그대로
    - "   " -> None
    - 그 외는 strip된 문자열
    """
    if v is None:
        return None
    if not isinstance(v, str):
        return v
    s = v.strip()
    return s or None


class UserBase(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    email: EmailStr

    @field_validator("email", mode="before")
    @classmethod
    def validate_email_strip(cls, v):
        if v is None:
            return v
        if isinstance(v, str):
            v = v.strip()
        if isinstance(v, str) and v == "":
            raise ValueError("email must not be blank")
        return v


class SettingsPatch(BaseModel):
    """
    [요청] PATCH /users/me/settings
    settings 부분 업데이트(merge)

    canonical key:
    - timezone
    - focus_mode
    - daily_goal_minutes

    legacy key:
    - daily_goal_min (호환용, 서버에서 daily_goal_minutes로 승격)
    """
    model_config = ConfigDict(str_strip_whitespace=True)

    settings: Dict[str, Any]

    @field_validator("settings")
    @classmethod
    def validate_settings(cls, v: Dict[str, Any]) -> Dict[str, Any]:
        if not isinstance(v, dict):
            raise ValueError("settings must be an object")

        allowed_keys = {"timezone", "focus_mode", "daily_goal_minutes", "daily_goal_min"}
        cleaned: Dict[str, Any] = {}

        for key, value in v.items():
            if not isinstance(key, str):
                continue

            normalized_key = key.strip()
            if normalized_key == "":
                continue

            if normalized_key not in allowed_keys:
                continue

            cleaned[normalized_key] = value

        if "daily_goal_minutes" in cleaned:
            value = cleaned["daily_goal_minutes"]
            if not isinstance(value, int) or isinstance(value, bool):
                raise ValueError("daily_goal_minutes must be an integer")

        if "daily_goal_min" in cleaned:
            value = cleaned["daily_goal_min"]
            if not isinstance(value, int) or isinstance(value, bool):
                raise ValueError("daily_goal_min must be an integer")

        if "timezone" in cleaned:
            value = cleaned["timezone"]
            if not isinstance(value, str) or value.strip() == "":
                raise ValueError("timezone must be a non-empty string")
            cleaned["timezone"] = value.strip()

        if "focus_mode" in cleaned:
            value = cleaned["focus_mode"]
            if not isinstance(value, str) or value.strip() == "":
                raise ValueError("focus_mode must be a non-empty string")
            cleaned["focus_mode"] = value.strip()

        return cleaned


class SettingsUpdate(SettingsPatch):
    """
    (호환용) SettingsPatch와 동일
    """
    pass


class FCMTokenAdd(BaseModel):
    """
    [요청] POST /users/me/fcm-tokens
    단일 FCM 토큰 추가
    """
    model_config = ConfigDict(str_strip_whitespace=True)

    token: str

    @field_validator("token")
    @classmethod
    def validate_token(cls, v: str) -> str:
        return _strip_and_reject_blank(v, "token")


class FCMTokenDelete(BaseModel):
    """
    [요청] DELETE /users/me/fcm-tokens
    - 표준 필드명은 token
    - fcm_token은 레거시 호환용
    - 전체 삭제는 허용하지 않음
    """
    model_config = ConfigDict(str_strip_whitespace=True)

    token: Optional[str] = None
    fcm_token: Optional[str] = None

    @field_validator("token", "fcm_token", mode="before")
    @classmethod
    def validate_optional_tokens(cls, v):
        return _strip_to_none(v)

    def resolved_token(self) -> Optional[str]:
        return self.token or self.fcm_token


class BlockedAppAdd(BaseModel):
    """
    [요청] POST /users/me/blocked-apps
    차단 앱 후보 추가
    """
    model_config = ConfigDict(str_strip_whitespace=True)

    app_name: str

    @field_validator("app_name")
    @classmethod
    def validate_app_name(cls, v: str) -> str:
        return _strip_and_reject_blank(v, "app_name")


class BlockedAppDelete(BaseModel):
    """
    [요청] DELETE /users/me/blocked-apps
    차단 앱 후보 제거
    """
    model_config = ConfigDict(str_strip_whitespace=True)

    app_name: str

    @field_validator("app_name")
    @classmethod
    def validate_app_name(cls, v: str) -> str:
        return _strip_and_reject_blank(v, "app_name")


class UserRead(UserBase):
    id: str
    created_at: datetime
    last_login_at: Optional[datetime] = None

    settings: Dict[str, Any] = Field(default_factory=dict)
    blocked_apps: List[str] = Field(default_factory=list)
    fcm_tokens: List[str] = Field(default_factory=list)

    model_config = {
        "from_attributes": True
    }


class SuccessMessage(BaseModel):
    success: bool = True
    message: str = "Operation successful"