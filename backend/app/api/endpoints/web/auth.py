from datetime import datetime, timezone
import os
from urllib.parse import urlencode, urlparse
from typing import Optional

from dotenv import load_dotenv
from fastapi import APIRouter, Request, HTTPException
from fastapi.responses import RedirectResponse
from pydantic import BaseModel
from authlib.integrations.starlette_client import OAuth, OAuthError
from google.oauth2 import id_token
from google.auth.transport import requests as google_requests

from app.db import mongo
from app.models.user import UserInDB
from app.core.security import create_access_token, create_refresh_token

load_dotenv()

router = APIRouter(prefix="/api/v1/auth", tags=["Auth (Web)"])

# 환경 변수 로드
GOOGLE_CLIENT_ID_DESKTOP = os.getenv("GOOGLE_CLIENT_ID")
WEB_GOOGLE_CLIENT_ID = os.getenv("WEB_GOOGLE_CLIENT_ID")
WEB_GOOGLE_CLIENT_SECRET = os.getenv("WEB_GOOGLE_CLIENT_SECRET")

BACKEND_PUBLIC_URL = os.getenv("BACKEND_PUBLIC_URL", "http://127.0.0.1:8000")
WEB_DASHBOARD_PUBLIC_URL = os.getenv("WEB_DASHBOARD_PUBLIC_URL")

# OAuth 설정
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
    try:
        allowed_clients = [WEB_GOOGLE_CLIENT_ID]
        if GOOGLE_CLIENT_ID_DESKTOP:
            allowed_clients.append(GOOGLE_CLIENT_ID_DESKTOP)

        # ID 토큰 검증
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

        user = await mongo.db.users.find_one({"email": email})

        if user:
            user_id_str = str(user["_id"])
            await mongo.db.users.update_one(
                {"_id": user["_id"]},
                {"$set": {"last_login_at": datetime.now(timezone.utc)}},
            )
        else:
            # UserInDB 스키마에 정의된 모든 필드(fcm_tokens 포함) 명시
            new_user = UserInDB(
                email=email,
                google_id=google_id,
                created_at=datetime.now(timezone.utc),
                last_login_at=datetime.now(timezone.utc),
                settings={},
                fcm_tokens=[],
                blocked_apps=[],
            )
            # Pydantic v2 방식 사용 (by_alias=True로 _id 처리)
            user_dict = new_user.model_dump(by_alias=True)
            insert_result = await mongo.db.users.insert_one(user_dict)
            user_id_str = str(insert_result.inserted_id)

        return TokenResponse(
            access_token=create_access_token(user_id_str),
            refresh_token=create_refresh_token(user_id_str),
        )

    except ValueError as e:
        raise HTTPException(status_code=400, detail=f"Invalid Google token: {str(e)}")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Internal Server Error: {str(e)}")

@router.get("/google/login")
async def google_login(request: Request, next: str | None = None):
    if not WEB_GOOGLE_CLIENT_ID or not WEB_GOOGLE_CLIENT_SECRET:
        raise HTTPException(status_code=500, detail="Web Google OAuth env is not configured")
    
    callback_uri = f"{BACKEND_PUBLIC_URL.rstrip('/')}/api/v1/auth/google/callback"
    if next:
        callback_uri = f"{callback_uri}?{urlencode({'next': next})}"
    
    return await oauth.google.authorize_redirect(request, callback_uri)

@router.get("/google/callback", response_model=TokenResponse)
async def google_callback(request: Request, next: str | None = None):
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
                fcm_tokens=[],
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