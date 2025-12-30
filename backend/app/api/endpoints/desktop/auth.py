from fastapi import APIRouter, Request, HTTPException
from fastapi.templating import Jinja2Templates 
from authlib.integrations.starlette_client import OAuth, OAuthError
from fastapi.responses import HTMLResponse
from jose import jwt
from datetime import datetime, timedelta, timezone
import os
import uuid
from dotenv import load_dotenv
from urllib.parse import urlencode

# mongo 모듈 자체를 import (db 변수 직접 import 시 None 문제 발생 방지)
from app.db import mongo
from app.models.user import UserInDB

load_dotenv()

router = APIRouter()
templates = Jinja2Templates(directory="app/templates")

# --- 환경 변수 로드 ---
GOOGLE_CLIENT_ID = os.getenv("GOOGLE_CLIENT_ID")
GOOGLE_CLIENT_SECRET = os.getenv("GOOGLE_CLIENT_SECRET")
SECRET_KEY = os.getenv("JWT_SECRET_KEY", "super-secret-key")
# [유지] 127.0.0.1 강제 고정 (로컬 테스트용)
BACKEND_PUBLIC_URL = os.getenv("BACKEND_PUBLIC_URL", "http://127.0.0.1:8000")

ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 60 * 24 # 1일
REFRESH_TOKEN_EXPIRE_DAYS = 30

# --- OAuth 설정 ---
oauth = OAuth()
oauth.register(
    name='google',
    client_id=GOOGLE_CLIENT_ID,
    client_secret=GOOGLE_CLIENT_SECRET,
    server_metadata_url='https://accounts.google.com/.well-known/openid-configuration',
    client_kwargs={'scope': 'openid email profile'}
)

# --- 헬퍼 함수: JWT 생성 ---
def create_token(data: dict, expires_delta: timedelta):
    to_encode = data.copy()
    expire = datetime.now(timezone.utc) + expires_delta
    to_encode.update({"exp": expire})
    return jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    
    
# --------------------------------------------------------------------------
# 1. 로그인 시작
# --------------------------------------------------------------------------
@router.get("/google/login")
async def login_via_google(request: Request):

    redirect_uri = f"{BACKEND_PUBLIC_URL}/api/v1/auth/google/callback"
    response = await oauth.google.authorize_redirect(request, redirect_uri)
    
    return response

# --------------------------------------------------------------------------
# 2. 콜백 처리
# --------------------------------------------------------------------------
@router.get("/google/callback")
async def auth_google_callback(request: Request):
    try:
        # A. 구글 토큰 및 정보 획득
        token = await oauth.google.authorize_access_token(request)
        
        user_info = token.get('userinfo')
        if not user_info:
            raise HTTPException(status_code=400, detail="Failed to get user info")
        
        email = user_info.get("email")

        # B. DB 연동 (mongo.db 사용)
        if mongo.db is None:
            raise HTTPException(status_code=500, detail="Database connection failed")

        user = await mongo.db.users.find_one({"email": email})
        
        if user:
            # 기존 유저: 로그인 시간 업데이트
            user_id = user["_id"]
            await mongo.db.users.update_one(
                {"_id": user_id},
                {"$set": {"last_login_at": datetime.now(timezone.utc)}}
            )
        else:
            user_id = str(uuid.uuid4())
            # 신규 유저: 생성
            #  모델 생성 시 누락된 필드(password_hash) 추가 및 ID 명시
            new_user = UserInDB(
                id=user_id,         # ConfigDict.populate_by_name = True 덕분에 id로 넣어도 됨
                email=email,
                created_at=datetime.now(timezone.utc),
                last_login_at=datetime.now(timezone.utc),
                settings={},
                blocked_apps=[]
            )
            # Pydantic v2 호환 (model_dump) 또는 v1 (dict)
            user_dict = new_user.dict(by_alias=True) if hasattr(new_user, 'dict') else new_user.model_dump(by_alias=True)
            await mongo.db.users.insert_one(user_dict)

        # C. 자체 JWT 발급
        access_token = create_token(
            data={"sub": user_id, "email": email, "type": "access"}, 
            expires_delta=timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
        )
        refresh_token = create_token(
            data={"sub": user_id, "type": "refresh"}, 
            expires_delta=timedelta(days=REFRESH_TOKEN_EXPIRE_DAYS)
        )

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
