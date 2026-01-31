# backend/app/api/endpoints/web/insight.py
import os
from google import genai
from fastapi import APIRouter, HTTPException
from app.crud import sessions as session_crud
from app.schemas.insight import InsightResponse
from app.utils.dummy_data import create_dummy_sessions

router = APIRouter(prefix="/insight", tags=["AI Insight"])
client = genai.Client(api_key=os.getenv("GEMINI_API_KEY"))

@router.get("/generate", response_model=InsightResponse)
async def generate_insight(
    user_id: str = "test_user_123",
    use_dummy: bool = False
):
    # 1. 데이터 준비 로직
    sessions = await session_crud.get_sessions(user_id, limit=5)
    if not sessions or use_dummy:
        await create_dummy_sessions(user_id)
        sessions = await session_crud.get_sessions(user_id, limit=5)

    input_data = "\n".join([
        f"작업: {s.task_id}, 시간: {s.duration}초, 상태: {s.status}, 방해: {s.interruption_count}회"
        for s in sessions
    ])

    try:
        # 2. 모델명을 로그에서 확인된 최신 버전으로 교체
        response = client.models.generate_content(
            model="gemini-2.5-flash", # 확인된 모델명 사용
            contents=f"다음 데이터를 분석하여 JSON 리포트를 작성하세요:\n{input_data}",
            config={
                "response_mime_type": "application/json",
                "response_schema": InsightResponse
            }
        )
        return response.parsed
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"AI 분석 실패: {str(e)}")