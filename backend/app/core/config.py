from pydantic_settings import BaseSettings
from typing import Optional

class Settings(BaseSettings):
    MONGO_URI: str
    MONGO_DB_NAME: str = "forcefocus"
    
    #  JWT 비밀키 (필수)
    JWT_SECRET_KEY: str
    
    #  구글 로그인 설정 (선택)
    GOOGLE_CLIENT_ID: Optional[str] = None
    GOOGLE_CLIENT_SECRET: Optional[str] = None

    class Config:
        # env_file = ".env"
        # .env에 정의되지 않은 변수가 있어도 무시하도록 설정 (오류 방지)
        extra = "ignore"

settings = Settings()