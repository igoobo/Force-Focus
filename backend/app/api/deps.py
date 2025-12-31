from fastapi import Depends, HTTPException, status
from fastapi.security import OAuth2PasswordBearer
from jose import jwt, JWTError
import os
from dotenv import load_dotenv

load_dotenv()

# 환경 변수 로드 (auth.py와 동일하게 맞춰야 함)
SECRET_KEY = os.getenv("JWT_SECRET_KEY", "super-secret-key")
ALGORITHM = "HS256"

# FastAPI가 스와거 문서에서 토큰 입력창을 보여주게 함
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="/api/v1/auth/google/login")

async def get_current_user_id(token: str = Depends(oauth2_scheme)) -> str:
    """
    JWT 토큰을 검증하고 user_id (sub)를 반환합니다.
    """
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Could not validate credentials",
        headers={"WWW-Authenticate": "Bearer"},
    )
    
    try:
        # 토큰 디코딩
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        user_id: str = payload.get("sub")
        
        if user_id is None:
            raise credentials_exception
            
        return user_id
        
    except JWTError:
        raise credentials_exception