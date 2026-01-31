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

    # 개인화 모델 저장 디렉토리 (기본값: 프로젝트 루트의 models_storage 폴더)
    # .env 파일에서 MODEL_STORAGE_DIR="/var/data/models" 와 같이 덮어쓸 수 있습니다.
    MODEL_STORAGE_DIR: str = "models_storage"

    class Config:
        env_file = ".env"
        # .env에 정의되지 않은 변수가 있어도 무시하도록 설정 (오류 방지)
        extra = "ignore"

settings = Settings()