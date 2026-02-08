# backend/app/schemas/schedule.py

from datetime import datetime, time, date
from typing import Optional, Annotated

from pydantic import BaseModel, Field, ConfigDict, field_validator

# --- 타입 별칭 (Pylance 경고 제거 + 검증 규칙 유지) ---
DayOfWeek = Annotated[int, Field(ge=0, le=6)]
DaysOfWeekCreate = Annotated[list[DayOfWeek], Field(min_length=1)]
DaysOfWeekUpdate = list[DayOfWeek]


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

class ScheduleCreate(BaseModel):
    """
    [요청] POST /schedules
    새로운 집중 세션 스케줄을 생성할 때 클라이언트가 보내는 데이터 구조입니다.
    """
    # ✅ 공통 strip 적용
    model_config = ConfigDict(str_strip_whitespace=True)

    task_id: Optional[str] = None
    name: str

    # ✅ 추가: yyyy-mm-dd
    start_date: Optional[date] = None
    end_date: Optional[date] = None

    start_time: time
    end_time: time

    # ✅ 추가: 설명(충분히 긴 문자열 가능)
    description: Optional[str] = None

    # 리스트 내부 요소는 0~6, 최소 1개 이상
    days_of_week: DaysOfWeekCreate

    @field_validator("name")
    @classmethod
    def validate_name(cls, v: str) -> str:
        return _strip_and_reject_blank(v, "name")

    @field_validator("task_id", mode="before")
    @classmethod
    def validate_task_id(cls, v):
        # Optional[str]는 "   " -> None으로 정규화
        return _strip_to_none(v)

    @field_validator("description", mode="before")
    @classmethod
    def validate_description(cls, v):
        # Optional[str]는 "   " -> None으로 정규화
        return _strip_to_none(v)


class ScheduleUpdate(BaseModel):
    """
    [요청] PUT /schedules/{schedule_id}
    기존 스케줄 정보를 업데이트할 때 클라이언트가 보내는 데이터 구조입니다.
    모든 필드는 선택 사항(Optional)입니다.
    """
    # ✅ 공통 strip 적용
    model_config = ConfigDict(str_strip_whitespace=True)

    task_id: Optional[str] = None
    name: Optional[str] = None

    # ✅ 추가
    start_date: Optional[date] = None
    end_date: Optional[date] = None
    description: Optional[str] = None

    start_time: Optional[time] = None
    end_time: Optional[time] = None
    days_of_week: Optional[DaysOfWeekUpdate] = None
    is_active: Optional[bool] = None

    @field_validator("name", mode="before")
    @classmethod
    def validate_name(cls, v):
        if v is None:
            return None
        return _strip_and_reject_blank(v, "name")

    @field_validator("task_id", mode="before")
    @classmethod
    def validate_task_id(cls, v):
        # Optional[str]는 "   " -> None으로 정규화
        return _strip_to_none(v)

    @field_validator("description", mode="before")
    @classmethod
    def validate_description(cls, v):
        # Optional[str]는 "   " -> None으로 정규화
        return _strip_to_none(v)


# --- API 응답(Response) 스키마 ---

class ScheduleRead(BaseModel):
    """
    [응답] GET /schedules, POST /schedules 등 조회/생성 성공 시
    데이터베이스에 저장된 스케줄 정보를 클라이언트에게 반환할 때의 데이터 구조입니다.
    """
    id: str
    user_id: str
    task_id: Optional[str] = None
    name: str

    # ✅ 추가
    start_date: Optional[date] = None
    end_date: Optional[date] = None
    description: Optional[str] = None

    start_time: time
    end_time: time
    days_of_week: list[int]
    created_at: datetime
    is_active: bool

    model_config = {
        "from_attributes": True
    }
