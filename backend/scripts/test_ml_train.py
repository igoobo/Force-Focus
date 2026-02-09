import asyncio
import sys
import os
from datetime import datetime, timezone

# 경로 설정
sys.path.append(os.path.join(os.path.dirname(__file__), ".."))

from app.db.mongo import connect_to_mongo, close_mongo_connection, get_db
from app.ml.train import train_user_model

TEST_USER_ID = "test_user_123"

async def generate_dummy_data(db):
    """테스트를 위한 더미 데이터 생성 (session_id 추가됨)"""
    print(f"Creating dummy data for {TEST_USER_ID}...")
    
    events = []
    # 1. 정상 작업 패턴
    for i in range(50):
        events.append({
            "user_id": TEST_USER_ID,
            "session_id": "session_test_01",  
            "timestamp": datetime.now(timezone.utc),
            "client_event_id": f"evt_{i}",
            "app_name": "Code.exe",
            "window_title": "main.rs - ForceFocus",
            "data": {
                "visible_windows": [
                    {"app_name": "Code.exe", "title": "main.rs", "rect": {"left":0, "top":0, "right":1920, "bottom":1080}}
                ],
                "meaningful_input_events": i * 10,
                "last_mouse_move_timestamp_ms": datetime.now().timestamp() * 1000
            }
        })
    
    # 2. 딴짓 패턴
    for i in range(10):
        events.append({
            "user_id": TEST_USER_ID,
            "session_id": "session_test_01", 
            "timestamp": datetime.now(timezone.utc),
            "client_event_id": f"evt_bad_{i}",
            "app_name": "chrome.exe",
            "window_title": "Netflix",
            "data": {
                "visible_windows": [],
                "meaningful_input_events": 0,
                "last_mouse_move_timestamp_ms": 0
            }
        })

    await db.events.insert_many(events)
    
    # 3. 피드백 생성
    await db.feedback.insert_one({
        "user_id": TEST_USER_ID,
        "client_event_id": "evt_bad_0",
        "feedback_type": "distraction_ignored",
        "timestamp": datetime.now(timezone.utc)
    })
    print("Dummy data inserted.")

async def main():
    print("1. Connecting to MongoDB...")
    await connect_to_mongo()
    db = get_db()

    # 기존 데이터가 꼬여있을 수 있으므로 삭제 후 재생성 추천 (테스트 환경)
    # await db.events.delete_many({"user_id": TEST_USER_ID}) 
    
    count = await db.events.count_documents({"user_id": TEST_USER_ID})
    if count < 50:
        await generate_dummy_data(db)
    else:
        # 기존 데이터에 session_id가 없는 경우를 대비해 업데이트
        await db.events.update_many(
            {"user_id": TEST_USER_ID, "session_id": {"$exists": False}},
            {"$set": {"session_id": "session_fix_01"}}
        )
    
    print(f"2. Starting Training for {TEST_USER_ID}...")
    result = await train_user_model(TEST_USER_ID)
    
    print("\n[Training Result]")
    print(result)
    
    if result["status"] == "success":
        print(f"\nModel saved at: backend/storage/models/{TEST_USER_ID}/{result['version']}/")
        
    await close_mongo_connection()

if __name__ == "__main__":
    asyncio.run(main())