# backend/app/db/mongo.py
from motor.motor_asyncio import AsyncIOMotorClient
from app.core.config import settings

client: AsyncIOMotorClient | None = None
db = None

async def connect_to_mongo():
    global client, db
    client = AsyncIOMotorClient(settings.MONGO_URI)
    db = client[settings.MONGO_DB_NAME]
    print("MongoDB Connected!")

async def close_mongo_connection():
    global client
    if client:
        client.close()
        print("MongoDB Connection Closed!")
