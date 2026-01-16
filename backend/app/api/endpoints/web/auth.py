from datetime import datetime, timezone
import os
from urllib.parse import urlencode, urlparse

from dotenv import load_dotenv
from fastapi import APIRouter, Request, HTTPException
from fastapi.responses import RedirectResponse
from pydantic import BaseModel
from authlib.integrations.starlette_client import OAuth, OAuthError
from google.oauth2 import id_token  # pip install google-auth
from google.auth.transport import requests as google_requests

from app.db import mongo
from app.models.user import UserInDB
from app.core.security import create_access_token, create_refresh_token

load_dotenv()

router = APIRouter(prefix="/api/v1/auth", tags=["Auth (Web)"])

# .env 파일의 변수명과 일치시킴
GOOGLE_CLIENT_ID_DESKTOP = os.getenv("GOOGLE_CLIENT_ID")
WEB_GOOGLE_CLIENT_ID = os.getenv("WEB_GOOGLE_CLIENT_ID")
WEB_GOOGLE_CLIENT_SECRET = os.getenv("WEB_GOOGLE_CLIENT_SECRET")

# URL 설정
BACKEND_PUBLIC_URL = os.getenv("BACKEND_PUBLIC_URL", "http://127.0.0.1:8000")
WEB_DASHBOARD_PUBLIC_URL = os.getenv("WEB_DASHBOARD_PUBLIC_URL")

# OAuth 설정 (웹용 ID/Secret 사용)
oauth = OAuth()
oauth.register(
    name="google",
    client_id=WEB_GOOGLE_CLIENT_ID,
    client_secret=WEB_GOOGLE_CLIENT_SECRET,
    server_metadata_url="https://accounts.google.com/.well-known/openid-configuration",
    client_kwargs={"scope": "openid email profile"},
)

class TokenResponse(BaseModel):
    access_token: str
    refresh_token: str
    token_type: str = "bearer"

class GoogleTokenBody(BaseModel):
    token: str

def _validate_next(next_url: str | None) -> str | None:
    """Open redirect 방지용 검증"""
    if not next_url:
        return None
    next_url = next_url.strip()
    if not next_url.startswith("/") and WEB_DASHBOARD_PUBLIC_URL:
        try:
            n = urlparse(next_url)
            w = urlparse(WEB_DASHBOARD_PUBLIC_URL)
            if (n.scheme, n.netloc) != (w.scheme, w.netloc):
                return None
        except:
            return None
    return next_url

@router.post("/google/verify", response_model=TokenResponse)
async def verify_google_token(body: GoogleTokenBody):
    """
    데스크탑과 웹 대시보드 ID를 모두 수용하여 ID 토큰 검증 및 DB 저장
    """
    try:
        # 두 클라이언트 ID 모두 허용 리스트에 추가
        allowed_clients = [WEB_GOOGLE_CLIENT_ID]
        if GOOGLE_CLIENT_ID_DESKTOP:
            allowed_clients.append(GOOGLE_CLIENT_ID_DESKTOP)

        # 구글 서버를 통한 토큰 검증
        idinfo = id_token.verify_oauth2_token(
            body.token, 
            google_requests.Request(), 
            audience=allowed_clients
        )

        email = idinfo.get("email")
        google_id = idinfo.get("sub")

        if not email:
            raise HTTPException(status_code=400, detail="Google token missing email")

        if mongo.db is None:
            raise HTTPException(status_code=500, detail="Database connection failed")

        # MongoDB 유저 처리 (Upsert 로직)
        user = await mongo.db.users.find_one({"email": email})

        if user:
            user_id_str = str(user["_id"])
            await mongo.db.users.update_one(
                {"_id": user["_id"]},
                {"$set": {"last_login_at": datetime.now(timezone.utc)}},
            )
        else:
            new_user = UserInDB(
                email=email,
                google_id=google_id,
                created_at=datetime.now(timezone.utc),
                last_login_at=datetime.now(timezone.utc),
                settings={},
                blocked_apps=[],
            )
            user_dict = new_user.model_dump(by_alias=True)
            insert_result = await mongo.db.users.insert_one(user_dict)
            user_id_str = str(insert_result.inserted_id)

        return TokenResponse(
            access_token=create_access_token(user_id_str),
            refresh_token=create_refresh_token(user_id_str),
        )

    except ValueError:
        raise HTTPException(status_code=400, detail="Invalid Google token")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@router.get("/google/login")
async def google_login(request: Request, next: str | None = None):
    """웹 대시보드 리다이렉트 로그인 시작"""
    if not WEB_GOOGLE_CLIENT_ID or not WEB_GOOGLE_CLIENT_SECRET:
        raise HTTPException(status_code=500, detail="Web Google OAuth env is not configured")
    
    callback_uri = f"{BACKEND_PUBLIC_URL.rstrip('/')}/api/v1/auth/google/callback"
    if next:
        callback_uri = f"{callback_uri}?{urlencode({'next': next})}"
    
    return await oauth.google.authorize_redirect(request, callback_uri)

@router.get("/google/callback", response_model=TokenResponse)
async def google_callback(request: Request, next: str | None = None):
    """Google OAuth 콜백 처리 및 리다이렉트"""
    try:
        token = await oauth.google.authorize_access_token(request)
        user_info = token.get("userinfo")
        email = user_info.get("email")
        
        user = await mongo.db.users.find_one({"email": email})
        if user:
            user_id_str = str(user["_id"])
            await mongo.db.users.update_one(
                {"_id": user["_id"]},
                {"$set": {"last_login_at": datetime.now(timezone.utc)}},
            )
        else:
            new_user = UserInDB(
                email=email,
                google_id=user_info.get("sub"),
                created_at=datetime.now(timezone.utc),
                last_login_at=datetime.now(timezone.utc),
                settings={},
                blocked_apps=[],
            )
            insert_result = await mongo.db.users.insert_one(new_user.model_dump(by_alias=True))
            user_id_str = str(insert_result.inserted_id)

        access_token = create_access_token(user_id_str)
        refresh_token = create_refresh_token(user_id_str)

        safe_next = _validate_next(next)
        if safe_next:
            fragment = urlencode({
                "access_token": access_token, 
                "refresh_token": refresh_token,
                "email": email
            })
            return RedirectResponse(url=f"{safe_next}#{fragment}")

        return TokenResponse(access_token=access_token, refresh_token=refresh_token)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))