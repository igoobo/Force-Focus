# backend/app/schemas/token.py

from pydantic import BaseModel

class TokenResponse(BaseModel):
    """
    Google 인증 성공 후 서버가 클라이언트에게 발급하는 토큰 응답 구조입니다.
    """
    access_token: str
    refresh_token: str
    token_type: str = "bearer"