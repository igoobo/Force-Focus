# backend/app/schemas/user.py

from datetime import datetime
from typing import Optional, List, Dict, Any

from pydantic import BaseModel, EmailStr, Field, ConfigDict, field_validator


# -------------------------
# 공백 방지 공통 유틸
# -------------------------
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
    # ✅ 공통 strip 적용
    model_config = ConfigDict(str_strip_whitespace=True)

    email: EmailStr

    # EmailStr도 "  a@b.com  " 같은 입력을 trim 후 검증되게 처리
    @field_validator("email", mode="before")
    @classmethod
    def validate_email_strip(cls, v):
        if v is None:
            return v
        if isinstance(v, str):
            v = v.strip()
        # 빈문자면 EmailStr 검증 전에 명확히 차단
        if isinstance(v, str) and v == "":
            raise ValueError("email must not be blank")
        return v


# ---------- 요청 스키마 (/users/me/*) ----------

class SettingsPatch(BaseModel):
    """
    [요청] PATCH /users/me/settings
    settings 부분 업데이트(merge)
    """
    # ✅ 공통 strip 적용
    model_config = ConfigDict(str_strip_whitespace=True)

    settings: Dict[str, Any]


# 기존 코드가 SettingsUpdate를 import해도 안 깨지게 alias 제공
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
    # ✅ 공통 strip 적용
    model_config = ConfigDict(str_strip_whitespace=True)

    token: str

    @field_validator("token")
    @classmethod
    def validate_token(cls, v: str) -> str:
        return _strip_and_reject_blank(v, "token")


class FCMTokenDelete(BaseModel):
    """
    [요청] DELETE /users/me/fcm-tokens
    - token 미지정 시 전체 삭제
    - {"token": "..."} 또는 {"fcm_token": "..."} 모두 허용(호환)
    """
    # ✅ 공통 strip 적용
    model_config = ConfigDict(str_strip_whitespace=True)

    # 표준 필드명은 token
    token: Optional[str] = None

    # 호환 필드명 (예전 코드가 payload.fcm_token을 쓰는 경우 대비)
    fcm_token: Optional[str] = None

    @field_validator("token", "fcm_token", mode="before")
    @classmethod
    def validate_optional_tokens(cls, v):
        # None은 허용(전체 삭제 케이스)
        # Optional[str]는 "   " -> None으로 정규화
        return _strip_to_none(v)

    def resolved_token(self) -> Optional[str]:
        """
        endpoint에서 어떤 필드를 쓰든 안전하게 하나로 합치기 위한 헬퍼
        """
        return self.token or self.fcm_token


class BlockedAppAdd(BaseModel):
    """
    [요청] POST /users/me/blocked-apps
    차단 앱 후보 추가
    """
    # ✅ 공통 strip 적용
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
    # ✅ 공통 strip 적용
    model_config = ConfigDict(str_strip_whitespace=True)

    app_name: str

    @field_validator("app_name")
    @classmethod
    def validate_app_name(cls, v: str) -> str:
        return _strip_and_reject_blank(v, "app_name")


# ---------- 응답 스키마 ----------

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
