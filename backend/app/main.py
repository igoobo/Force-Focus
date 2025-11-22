# main.py
from fastapi import FastAPI
from app.api.endpoints.web import tasks, schedules
from app.db.mongo import connect_to_mongo, close_mongo_connection

app = FastAPI(title="Force Focus Backend")

@app.on_event("startup")
async def startup_db():
    await connect_to_mongo()

@app.on_event("shutdown")
async def shutdown_db():
    await close_mongo_connection()

@app.get("/")
async def read_root():
    return {"message": "Backend is running!"}

# 나중에 API 라우터들을 여기에 include 합니다.
# from app.api.endpoints import users, sessions # 예시
# app.include_router(users.router, prefix="/users", tags=["users"])

app.include_router(tasks.router, prefix="/tasks", tags=["tasks"])
app.include_router(schedules.router, prefix="/schedules", tags=["schedules"])
