# 파일 위치: backend/app/schemas/rule.py

from pydantic import BaseModel
from typing import Dict

# --- API 응답(Response) 스키마 ---
class AppRuleRead(BaseModel):
    """
    [응답] GET /rules/heuristics (가칭)
    MVP 단계에서 데스크탑 에이전트가 사용할 휴리스틱 규칙을 반환할 때의 데이터 구조입니다.
    """
    rule_name: str
    condition: Dict[str, any]
    action: Dict[str, any]
    priority: int

    class Config:
        orm_mode = True