# backend/app/core/security.py

from datetime import datetime, timedelta, timezone
from jose import jwt
from typing import Any, Union
import os
from dotenv import load_dotenv

load_dotenv()

SECRET_KEY = os.getenv("JWT_SECRET_KEY")
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 60 * 24  # 1일
REFRESH_TOKEN_EXPIRE_DAYS = 30

if not SECRET_KEY:
    raise RuntimeError("JWT_SECRET_KEY is not configured")


def create_access_token(subject: Union[str, Any]) -> str:
    """
    Access Token 생성 (유효기간 짧음)
    """
    expire = datetime.now(timezone.utc) + timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
    to_encode = {
        "exp": expire,
        "sub": str(subject),
        "type": "access",
    }
    return jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)


def create_refresh_token(subject: Union[str, Any]) -> str:
    """
    Refresh Token 생성 (유효기간 김)
    """
    expire = datetime.now(timezone.utc) + timedelta(days=REFRESH_TOKEN_EXPIRE_DAYS)
    to_encode = {
        "exp": expire,
        "sub": str(subject),
        "type": "refresh",
    }
    return jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)