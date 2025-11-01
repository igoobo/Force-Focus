# 파일 위치: backend/app/models/rule.py

from pydantic import BaseModel, Field
from typing import Dict

class AppRuleInDB(BaseModel):
    """
    MongoDB의 'app_rules' 컬렉션에 저장되는 완전한 형태의 데이터 모델입니다.
    """
    id: str = Field(..., alias="_id") # MongoDB의 '_id'를 'id'로 매핑
    rule_name: str
    condition: Dict[str, any]
    action: Dict[str, any]
    priority: int

    class Config:
        allow_population_by_field_name = True
        orm_mode = True