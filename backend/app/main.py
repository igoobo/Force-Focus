# main.py
import os
from contextlib import asynccontextmanager
from fastapi import FastAPI
from starlette.middleware.sessions import SessionMiddleware 
from fastapi.middleware.cors import CORSMiddleware

from app.api.endpoints.web import tasks, schedules
from app.api.endpoints.desktop import auth as desktop_auth
from app.db.mongo import connect_to_mongo, close_mongo_connection
from dotenv import load_dotenv

load_dotenv()

# [설정] 환경 구분 (기본값: development)
# 프로덕션 배포 시 .env에 ENVIRONMENT=production 으로 설정해야 합니다.
ENVIRONMENT = os.getenv("ENVIRONMENT", "development")
IS_PRODUCTION = ENVIRONMENT == "production"

# [설정] 로컬 개발용 HTTP 허용 (authlib)
# 프로덕션(HTTPS)에서는 절대 이 설정을 켜면 안 됩니다. (보안 취약점)
if not IS_PRODUCTION:
    os.environ['OAUTHLIB_INSECURE_TRANSPORT'] = '1'
    print(f"⚠️ Running in {ENVIRONMENT} mode. Insecure transport enabled.")


# [수명 주기 관리] DB 연결 및 해제
@asynccontextmanager
async def lifespan(app: FastAPI):
    # Startup 로직
    await connect_to_mongo()
    yield
    # Shutdown 로직
    await close_mongo_connection()

# lifespan 파라미터로 수명 주기 핸들러 등록
app = FastAPI(title="Force Focus Backend", lifespan=lifespan)

#  SessionMiddleware 등록 (OAuth용)
SECRET_KEY = os.getenv("JWT_SECRET_KEY", "default-insecure-secret-key")

# --- 미들웨어 설정 ---

# 1. CORS: 프론트엔드 및 데스크톱 앱 접근 허용
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:1420", 
        "http://127.0.0.1:1420", 
        "http://127.0.0.1:8000", 
        "http://localhost:8000"
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# 2. Session: OAuth 인증 상태 관리 (쿠키)
# 프로덕션 환경에 따른 보안 설정 분기
app.add_middleware(
    SessionMiddleware, 
    secret_key=SECRET_KEY,
    # [핵심] 프로덕션(HTTPS)에서는 True, 개발(HTTP)에서는 False
    https_only=IS_PRODUCTION,  
    # 리다이렉트 흐름을 위해 lax 유지 (Strict는 리다이렉트 시 쿠키 미전송)
    same_site="lax", 
    path="/",
    max_age=3600,
    domain=None 
)



@app.get("/")
async def read_root():
    return {"message": "Backend is running!"}

# 웹 대시보드 API
app.include_router(tasks.router, prefix="/tasks", tags=["tasks"])
app.include_router(schedules.router, prefix="/schedules", tags=["schedules"])

# 데스크톱 에이전트 API (인증)
app.include_router(desktop_auth.router, prefix="/api/v1/auth", tags=["auth"])