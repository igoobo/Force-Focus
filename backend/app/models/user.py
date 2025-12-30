# 파일 위치: backend/app/models/user.py (또는 schemas/user.py)
from pydantic import BaseModel, Field, EmailStr, ConfigDict
from datetime import datetime
from typing import Optional, List, Dict, Any

class UserInDB(BaseModel):
    """
    MongoDB에 저장되는 완전한 형태의 User 모델입니다.
    데이터 모델링 표의 내용을 코드로 구체화한 것입니다.
    """
    # MongoDB의 고유 ID인 "_id"를 "id" 필드로 사용하기 위한 설정입니다.
    id: str = Field(..., alias="_id")
    
    # EmailStr 타입을 사용하여 이메일 형식을 자동으로 검증합니다.
    email: EmailStr

    # # OAuth 로그인 정보 (비밀번호 제거, 제공자 추가)
    # provider: str = "google" # "google", "apple", "local" 등
    # picture: Optional[str] = None # 구글 프로필 사진 URL

    # 표의 'timestamp'는 Python의 'datetime' 객체로 매핑됩니다.
    created_at: datetime = Field(default_factory=datetime.now)
    last_login_at: Optional[datetime] = None
    
   
    # 타입 힌트 강화: Any -> 구체적인 타입 권장하지만, 유연성을 위해 Any 유지
    settings: Dict[str, Any] = Field(default_factory=dict)
    
    #  다중 기기 지원을 위해 List[str]로 변경 (확장성 고려)
    fcm_tokens: List[str] = Field(default_factory=list)
    
    # 차단된 앱 목록
    blocked_apps: List[str] = Field(default_factory=list)

    # [수정] Pydantic V2 설정 방식
    model_config = ConfigDict(
        populate_by_name=True,       # 'id'라는 이름으로 값을 넣어도 '_id' 필드에 할당 허용
        from_attributes=True,        # ORM 객체(Dict 등)로부터 데이터 로드 허용 (구 orm_mode)
        arbitrary_types_allowed=True # datetime 등 복잡한 타입 허용
    )