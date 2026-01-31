# backend/app/api/endpoints/web/insight.py
import os
import google.generativeai as genai
from fastapi import APIRouter, Depends, HTTPException
from app.api.deps import get_current_user_id
from app.crud import sessions as session_crud
from app.schemas.insight import InsightResponse
from app.utils.dummy_data import create_dummy_sessions # 유틸리티 로드

router = APIRouter(prefix="/insight", tags=["AI Insight"])

# Gemini 초기화
genai.configure(api_key=os.getenv("GEMINI_API_KEY"))
model = genai.GenerativeModel('gemini-1.5-flash')

@router.get("/generate", response_model=InsightResponse)
async def generate_insight(
    user_id: str = Depends(get_current_user_id),
    use_dummy: bool = False # 쿼리 파라미터로 더미 사용 여부 선택 가능
):
    # 1. 데이터 조회
    sessions = await session_crud.get_sessions(user_id, limit=5)
    
    # 2. 데이터가 없거나 use_dummy가 True일 때 가짜 데이터 생성
    if not sessions or use_dummy:
        await create_dummy_sessions(user_id)
        sessions = await session_crud.get_sessions(user_id, limit=5) # 다시 조회

    # 3. AI 분석 컨텍스트 구성
    input_data = "\n".join([
        f"작업: {s.task_id}, 시간: {s.duration}초, 상태: {s.status}, 방해: {s.interruption_count}회"
        for s in sessions
    ])

    try:
        response = model.generate_content(
            f"다음 데이터를 분석하여 Feedback.jsx용 JSON 리포트를 작성하세요:\n{input_data}",
            generation_config={
                "response_mime_type": "application/json",
                "response_schema": InsightResponse
            }
        )
        return InsightResponse.model_validate_json(response.text)
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"AI 분석 실패: {str(e)}")