# backend/app/main.py

import os
from contextlib import asynccontextmanager

from dotenv import load_dotenv
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from starlette.middleware.sessions import SessionMiddleware

from app.db.mongo import connect_to_mongo, close_mongo_connection

# -------------------------
# Env
# -------------------------
load_dotenv()

ENVIRONMENT = os.getenv("ENVIRONMENT", "development")
IS_PRODUCTION = ENVIRONMENT == "production"

if not IS_PRODUCTION:
    os.environ["OAUTHLIB_INSECURE_TRANSPORT"] = "1"
    print(f"⚠️ Running in {ENVIRONMENT} mode. Insecure transport enabled.")

# -------------------------
# Lifespan (DB lifecycle)
# -------------------------
@asynccontextmanager
async def lifespan(app: FastAPI):
    await connect_to_mongo()
    yield
    await close_mongo_connection()

app = FastAPI(
    title="Force Focus Backend",
    lifespan=lifespan,
)

# -------------------------
# Middleware
# -------------------------
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:1420",
        "http://127.0.0.1:1420",
        "http://localhost:3000",
        "http://127.0.0.1:3000",
        "http://localhost:8000",
        "http://127.0.0.1:8000",
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

SESSION_SECRET = (
    os.getenv("SESSION_SECRET_KEY")
    or os.getenv("JWT_SECRET_KEY")
    or "default-insecure-secret-key"
)

app.add_middleware(
    SessionMiddleware,
    secret_key=SESSION_SECRET,
    https_only=IS_PRODUCTION,
    same_site="lax",
    max_age=3600,
)

# -------------------------
# Health check
# -------------------------
@app.get("/")
async def read_root():
    return {"message": "Backend is running!"}

# -------------------------
# Routers
# -------------------------

# Web
from app.api.endpoints.web import (
    auth as web_auth,
    tasks,
    schedules,
    sessions,
    events as web_events,
    feedback,
)

# User
from app.api.endpoints.user import me

# Desktop
from app.api.endpoints.desktop import (
    auth as desktop_auth,
    events as desktop_events,
)

# Web Auth
app.include_router(web_auth.router)

# User
app.include_router(me.router)

# Web Core
app.include_router(tasks.router)
app.include_router(schedules.router)
app.include_router(sessions.router)
app.include_router(web_events.router)
app.include_router(feedback.router)

# Desktop APIs
app.include_router(desktop_auth.router, prefix="/api/v1/auth/desktop", tags=["auth-desktop"])
app.include_router(desktop_events.router, prefix="/api/v1/events", tags=["events"])
