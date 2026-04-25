from datetime import datetime, timezone
import os
from urllib.parse import urlencode, urlparse

from dotenv import load_dotenv
from fastapi import APIRouter, Request, HTTPException, status
from fastapi.responses import RedirectResponse
from pydantic import BaseModel
from authlib.integrations.starlette_client import OAuth, OAuthError
from google.oauth2 import id_token
from google.auth.transport import requests as google_requests

from app.db import mongo
from app.models.user import UserInDB
from app.core.security import create_access_token, create_refresh_token
from app.schemas.token import TokenResponse

load_dotenv()

router = APIRouter(prefix="/api/v1/auth", tags=["Auth (Web)"])

GOOGLE_CLIENT_ID_DESKTOP = os.getenv("GOOGLE_CLIENT_ID")
WEB_GOOGLE_CLIENT_ID = os.getenv("WEB_GOOGLE_CLIENT_ID")
WEB_GOOGLE_CLIENT_SECRET = os.getenv("WEB_GOOGLE_CLIENT_SECRET")

BACKEND_PUBLIC_URL = os.getenv("BACKEND_PUBLIC_URL", "http://127.0.0.1:8000")
WEB_DASHBOARD_PUBLIC_URL = os.getenv("WEB_DASHBOARD_PUBLIC_URL")

oauth = OAuth()
oauth.register(
    name="google",
    client_id=WEB_GOOGLE_CLIENT_ID,
    client_secret=WEB_GOOGLE_CLIENT_SECRET,
    server_metadata_url="https://accounts.google.com/.well-known/openid-configuration",
    client_kwargs={"scope": "openid email profile"},
)


class GoogleTokenBody(BaseModel):
    token: str


def _ensure_db_connected() -> None:
    if mongo.db is None:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="Database connection failed",
        )


def _validate_next(next_url: str | None) -> str | None:
    if not next_url:
        return None
    
    next_url = next_url.strip()
    if not next_url:
        return None

    parsed_next = urlparse(next_url)

    # 상대 경로는 허용
    if next_url.startswith("/"):
        return next_url

    # WEB_DASHBOARD_PUBLIC_URL이 없으면 절대 URL은 허용하지 않음
    if not WEB_DASHBOARD_PUBLIC_URL:
        return None

    parsed_dashboard = urlparse(WEB_DASHBOARD_PUBLIC_URL)

    if (parsed_next.scheme, parsed_next.netloc) != (
        parsed_dashboard.scheme,
        parsed_dashboard.netloc,
    ):
        return None

    return next_url


async def _get_or_create_user(email: str, google_id: str | None) -> str:
    _ensure_db_connected()

    user = await mongo.db.users.find_one({"email": email})
    now = datetime.now(timezone.utc)

    if user:
        await mongo.db.users.update_one(
            {"_id": user["_id"]},
            {"$set": {"last_login_at": now}},
        )
        return str(user["_id"])

    new_user = UserInDB(
        email=email,
        google_id=google_id,
        created_at=now,
        last_login_at=now,
        settings={},
        fcm_tokens=[],
        blocked_apps=[],
    )
    insert_result = await mongo.db.users.insert_one(new_user.model_dump(by_alias=True))
    return str(insert_result.inserted_id)


@router.post("/google/verify", response_model=TokenResponse)
async def verify_google_token(body: GoogleTokenBody):
    if not WEB_GOOGLE_CLIENT_ID:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="WEB_GOOGLE_CLIENT_ID is not configured",
        )

    try:
        idinfo = id_token.verify_oauth2_token(
            body.token,
            google_requests.Request(),
            audience=WEB_GOOGLE_CLIENT_ID,
        )

        email = idinfo.get("email")
        google_id = idinfo.get("sub")

        if not email:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Google token missing email",
            )

        user_id_str = await _get_or_create_user(email=email, google_id=google_id)

        return TokenResponse(
            access_token=create_access_token(user_id_str),
            refresh_token=create_refresh_token(user_id_str),
            token_type="bearer",
        )

    except HTTPException:
        raise
    except ValueError:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Invalid Google token",
        )
    except Exception:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="Failed to verify Google token",
        )


@router.get("/google/login")
async def google_login(request: Request, next: str | None = None):
    if not WEB_GOOGLE_CLIENT_ID or not WEB_GOOGLE_CLIENT_SECRET:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="Web Google OAuth env is not configured",
        )

    safe_next = _validate_next(next)

    callback_uri = f"{BACKEND_PUBLIC_URL.rstrip('/')}/api/v1/auth/google/callback"
    if safe_next:
        callback_uri = f"{callback_uri}?{urlencode({'next': safe_next})}"

    return await oauth.google.authorize_redirect(request, callback_uri)


@router.get("/google/callback")
async def google_callback(request: Request, next: str | None = None):
    try:
        _ensure_db_connected()

        token = await oauth.google.authorize_access_token(request)
        user_info = token.get("userinfo")

        if not user_info:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Invalid OAuth response",
            )

        email = user_info.get("email")
        google_id = user_info.get("sub")

        if not email:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="OAuth response missing email",
            )

        user_id_str = await _get_or_create_user(email=email, google_id=google_id)

        access_token = create_access_token(user_id_str)
        refresh_token = create_refresh_token(user_id_str)

        safe_next = _validate_next(next)
        if safe_next:
            fragment = urlencode(
                {
                    "access_token": access_token,
                    "refresh_token": refresh_token,
                    "email": email,
                }
            )
            return RedirectResponse(url=f"{safe_next}#{fragment}")

        return TokenResponse(
            access_token=access_token,
            refresh_token=refresh_token,
            token_type="bearer",
        )

    except HTTPException:
        raise
    except OAuthError:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Google OAuth failed",
        )
    except Exception:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="Failed to complete Google login",
        )