from fastapi import APIRouter, Request, HTTPException
from fastapi.templating import Jinja2Templates 
from authlib.integrations.starlette_client import OAuth, OAuthError
from fastapi.responses import HTMLResponse
from datetime import datetime, timedelta, timezone

import os
from dotenv import load_dotenv
from urllib.parse import urlencode

# mongo 모듈 자체를 import (db 변수 직접 import 시 None 문제 발생 방지)
from app.db import mongo
from app.models.user import UserInDB
from app.core.security import create_access_token, create_refresh_token

load_dotenv()

router = APIRouter()
templates = Jinja2Templates(directory="app/templates")

# --- 환경 변수 로드 ---
GOOGLE_CLIENT_ID = os.getenv("GOOGLE_CLIENT_ID")
GOOGLE_CLIENT_SECRET = os.getenv("GOOGLE_CLIENT_SECRET")
# [유지] 127.0.0.1 강제 고정 (로컬 테스트용)
BACKEND_PUBLIC_URL = os.getenv("BACKEND_PUBLIC_URL", "http://127.0.0.1:8000")

# --- OAuth 설정 ---
oauth = OAuth()
oauth.register(
    name='google',
    client_id=GOOGLE_CLIENT_ID,
    client_secret=GOOGLE_CLIENT_SECRET,
    server_metadata_url='https://accounts.google.com/.well-known/openid-configuration',
    client_kwargs={'scope': 'openid email profile'}
)

# --------------------------------------------------------------------------
# 1. 로그인 시작
# --------------------------------------------------------------------------
@router.get("/google/login")
async def login_via_google(request: Request):

    redirect_uri = f"{BACKEND_PUBLIC_URL}/api/v1/auth/desktop/google/callback"
    response = await oauth.google.authorize_redirect(request, redirect_uri)
    
    return response

# --------------------------------------------------------------------------
# 2. 콜백 처리
# --------------------------------------------------------------------------
@router.get("/google/callback")
async def auth_google_callback(request: Request):
    try:
        # A. 구글 토큰 및 정보 획득
        # authlib가 세션에서 자동으로 redirect_uri를 가져오도록
        # (수동으로 넘기면 'multiple values' 에러 발생)
        token = await oauth.google.authorize_access_token(request)
        
        user_info = token.get('userinfo')
        if not user_info:
            raise HTTPException(status_code=400, detail="Failed to get user info")
        
        email = user_info.get("email")
        google_sub = user_info.get("sub")

        # B. DB 연동 (mongo.db 사용)
        if mongo.db is None:
            raise HTTPException(status_code=500, detail="Database connection failed")

        user = await mongo.db.users.find_one({"email": email})
        # 변수 초기화 (스코프 문제 방지)
        user_id_str = ""
    
        if user:
            # 기존 유저: 로그인 시간 업데이트
            user_id_obj = user["_id"]
            await mongo.db.users.update_one(
                {"_id": user_id_obj},
                {"$set": {"last_login_at": datetime.now(timezone.utc)}}
            )
            # JWT 생성을 위해 ObjectId -> str 변환 (변수 할당)
            user_id_str = str(user_id_obj)
        else:
            # [신규 유저] 생성
            new_user = UserInDB(
                email=email,
                google_id=google_sub,
                created_at=datetime.now(timezone.utc),
                last_login_at=datetime.now(timezone.utc)
                # settings, fcm_tokens, blocked_apps는 default_factory로 자동 생성
            )
            result = await mongo.db.users.insert_one(new_user.model_dump(by_alias=True))
            user_id_str = str(result.inserted_id)

        # C. 공통 모듈 사용하여 토큰 발급
        if not user_id_str:
             raise HTTPException(status_code=500, detail="User ID generation failed")
        
        access_token = create_access_token(user_id_str)
        refresh_token = create_refresh_token(user_id_str)

        # D. 데스크톱 앱 깨우기 (Deep Link Redirect)
        # URL 파라미터 인코딩 적용 (특수문자 등으로 인한 파싱 오류 방지)
        query_params = {
            "access_token": access_token,
            "refresh_token": refresh_token,
            "email": email
        }
        query_string = urlencode(query_params)
        
        # 딥 링크 URL
        deep_link_url = f"force-focus://auth/callback?{query_string}"

        # HTML 응답 반환 (자동 실행 스크립트 + 수동 버튼 포함)
        return templates.TemplateResponse(
            "desktop_login_success.html", 
            {"request": request, "deep_link": deep_link_url, "email": email}
        )

    except OAuthError as e:
        return HTMLResponse(content=f"<h1>Authentication Failed</h1><p>{str(e)}</p>", status_code=400)
    except Exception as e:
        return HTMLResponse(content=f"<h1>Server Error</h1><p>{str(e)}</p>", status_code=500)