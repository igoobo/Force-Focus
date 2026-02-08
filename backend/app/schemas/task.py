# backend/app/schemas/task.py

from datetime import datetime
from typing import Optional

from pydantic import BaseModel, ConfigDict, field_validator


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


# --- API 요청(Request) 스키마 ---
class TaskCreate(BaseModel):
    """
    [요청] POST /tasks
    새로운 할 일을 생성할 때 클라이언트가 보내는 데이터 구조입니다.
    user_id는 인증 토큰에서 가져오므로 클라이언트가 보낼 필요가 없습니다.
    """
    # ✅ 공통 strip 적용
    model_config = ConfigDict(str_strip_whitespace=True)

    name: str
    description: Optional[str] = None
    due_date: Optional[datetime] = None
    target_executable: Optional[str] = None
    target_arguments: Optional[str] = None

    # ✅ 추가: 기본 제공/사용자 추가 구분
    # - 사용자 생성 POST /tasks는 기본적으로 custom으로 보는 게 자연스러워서 True 기본값 권장
    isCustom: bool = True

    @field_validator("name")
    @classmethod
    def validate_name(cls, v: str) -> str:
        return _strip_and_reject_blank(v, "name")

    @field_validator("description", "target_executable", "target_arguments", mode="before")
    @classmethod
    def validate_optional_strings(cls, v):
        # Optional[str]에서 "   "는 None으로 정규화
        return _strip_to_none(v)


class TaskUpdate(BaseModel):
    """
    [요청] PUT /tasks/{task_id}
    기존 할 일 정보를 업데이트할 때 클라이언트가 보내는 데이터 구조입니다.
    모든 필드는 선택 사항(Optional)입니다.
    """
    # ✅ 공통 strip 적용
    model_config = ConfigDict(str_strip_whitespace=True)

    name: Optional[str] = None
    description: Optional[str] = None
    due_date: Optional[datetime] = None
    status: Optional[str] = None  # "pending", "completed" 등으로 변경
    linked_session_id: Optional[str] = None
    target_executable: Optional[str] = None
    target_arguments: Optional[str] = None
    isCustom: Optional[bool] = None

    @field_validator("name", mode="before")
    @classmethod
    def validate_name(cls, v):
        if v is None:
            return None
        return _strip_and_reject_blank(v, "name")

    @field_validator(
        "description",
        "status",
        "linked_session_id",
        "target_executable",
        "target_arguments",
        mode="before",
    )
    @classmethod
    def validate_optional_strings(cls, v):
        # Optional[str]에서 "   "는 None으로 정규화
        return _strip_to_none(v)


# --- API 응답(Response) 스키마 ---
class TaskRead(BaseModel):
    """
    [응답] GET /tasks/{task_id}, POST /tasks 등 조회/생성 성공 시
    데이터베이스에 저장된 할 일 정보를 클라이언트에게 반환할 때의 데이터 구조입니다.
    """
    id: str
    user_id: str
    name: str
    description: Optional[str] = None
    created_at: datetime
    due_date: Optional[datetime] = None
    status: str
    linked_session_id: Optional[str] = None
    target_executable: Optional[str] = None
    target_arguments: Optional[str] = None
    isCustom: bool

    model_config = {
        "from_attributes": True
    }
