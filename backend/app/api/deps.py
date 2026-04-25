# backend/app/api/deps.py

from fastapi import Depends, HTTPException, status
from fastapi.security import OAuth2PasswordBearer
from jose import jwt, JWTError
import os
from dotenv import load_dotenv

load_dotenv()

SECRET_KEY = os.getenv("JWT_SECRET_KEY")
ALGORITHM = "HS256"

if not SECRET_KEY:
    raise RuntimeError("JWT_SECRET_KEY is not configured")

# Swagger에서 Bearer 토큰 입력창을 제공하기 위한 설정
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="/api/v1/auth/google/verify")


async def get_current_user_id(token: str = Depends(oauth2_scheme)) -> str:
    """
    JWT 토큰을 검증하고 access token의 user_id(sub)를 반환합니다.
    """
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Could not validate credentials",
        headers={"WWW-Authenticate": "Bearer"},
    )

    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])

        token_type = payload.get("type")
        user_id = payload.get("sub")

        if token_type != "access":
            raise credentials_exception

        if not user_id:
            raise credentials_exception

        return user_id

    except JWTError:
        raise credentials_exception