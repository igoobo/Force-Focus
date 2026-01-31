import os
from fastapi import APIRouter, Depends
import google.generativeai as genai
from app.api.deps import get_current_user_id
from app.crud import sessions as session_crud
from app.schemas.insight import InsightResponse

router = APIRouter(prefix="/insight", tags=["AI Insight"])

# Gemini 초기화 (JSON 모드 및 스키마 적용)
genai.configure(api_key=os.getenv("GEMINI_API_KEY"))
model = genai.GenerativeModel(
    model_name='gemini-1.5-flash',
    system_instruction="당신은 전문 생산성 분석가입니다. 사용자의 앱 사용 패턴과 세션 데이터를 분석하여 'Feedback.jsx' UI에 출력될 정교한 리포트를 JSON으로 생성하십시오."
)

@router.get("/generate", response_model=InsightResponse)
async def get_ai_feedback(user_id: str = Depends(get_current_user_id)):
    # 최근 세션과 관련 앱 사용 로그(태깅 데이터) 조회
    # (session_schema.py의 SessionRead 형식을 따름)
    sessions = await session_crud.get_sessions(user_id, limit=5)
    
    # AI에게 넘길 데이터 문자열화
    input_context = "\n".join([
        f"세션: {s.task_id}, 소요: {s.duration}초, 상태: {s.status}, 방해횟수: {s.interruption_count}"
        for s in sessions
    ])

    prompt = f"""
    아래 세션 데이터를 바탕으로 생산성 리포트를 작성하세요:
    {input_context}

    [작성 가이드라인]
    1. 톤: 분석적이며 전문적인 코칭 톤 (예: '인지적 임계점', '디폴트 모드 네트워크' 등 용어 사용)
    2. 내용: 데이터 기반의 구체적 수치 제시
    3. 형식: 반드시 제공된 JSON 스키마를 준수할 것
    """

    response = model.generate_content(
        prompt,
        generation_config={
            "response_mime_type": "application/json",
            "response_schema": InsightResponse
        }
    )
    
    return InsightResponse.model_validate_json(response.text)