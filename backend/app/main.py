# backend/app/main.py
from fastapi import FastAPI
from app.api.endpoints.web import tasks, schedules

app = FastAPI(title="Force Focus Backend")

@app.get("/")
async def read_root():
    return {"message": "Backend is running!"}

# 나중에 API 라우터들을 여기에 include 합니다.
# from app.api.endpoints import users, sessions # 예시
# app.include_router(users.router, prefix="/users", tags=["users"])

app.include_router(tasks.router, prefix="/tasks", tags=["tasks"])
app.include_router(schedules.router, prefix="/schedules", tags=["schedules"])
