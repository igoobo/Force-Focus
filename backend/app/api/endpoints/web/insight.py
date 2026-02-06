import os
from typing import Optional
from fastapi import APIRouter, HTTPException, Depends, Query
from google import genai
from datetime import datetime

from app.api.deps import get_current_user_id
from app.crud import sessions as session_crud
from app.crud import events as event_crud
from app.schemas.insight import InsightResponse

router = APIRouter(prefix="/insight", tags=["AI Insight"])

# Gemini 클라이언트 설정
client = genai.Client(api_key=os.getenv("GEMINI_API_KEY"))

async def get_session_context_for_llm(user_id: str, session_id: str) -> str:
    """
    특정 세션의 이벤트들을 LLM이 분석하기 좋은 텍스트로 변환합니다.
    """
    # 세션 내 이벤트를 최대 300개까지 가져옵니다. (필요시 조절)
    events = await event_crud.get_events(user_id=user_id, session_id=session_id, limit=300)
    
    if not events:
        return "기록된 세부 활동 로그가 없습니다."

    # 시간순 정렬 (과거 -> 현재)
    events.reverse()
    
    event_logs = []
    for e in events:
        time_str = e.timestamp.strftime("%H:%M:%S")
        log_line = f"[{time_str}] 앱: {e.app_name or '알 수 없음'}, 제목: {e.window_title or '정보 없음'}"
        event_logs.append(log_line)

    return "\n".join(event_logs)

@router.get("/analyze/{session_id}", response_model=InsightResponse)
async def analyze_session_insight(
    session_id: str,
    user_id: str = Depends(get_current_user_id)
):
    """
    특정 세션 ID를 받아 해당 세션의 모든 이벤트를 분석하고 JSON 리포트를 생성합니다.
    """
    # 1. 세션 기본 정보 조회
    session = await session_crud.get_session(session_id)
    if not session:
        raise HTTPException(status_code=404, detail="세션 정보를 찾을 수 없습니다.")
    
    if session.user_id != user_id:
        raise HTTPException(status_code=403, detail="해당 세션에 접근할 권한이 없습니다.")

    # 2. 세션 내 상세 이벤트 로그 추출
    event_context = await get_session_context_for_llm(user_id, session_id)

    # 3. LLM 프롬프트 구성
    # InsightResponse 스키마의 필드 설명을 기반으로 페르소나를 부여합니다.
    prompt = f"""
    당신은 전문적인 생산성 분석가입니다. 아래 제공된 사용자의 작업 세션 데이터를 분석하여 상세 리포트를 작성하세요.
    응답은 반드시 'InsightResponse' JSON 구조를 엄격히 따라야 합니다.

    [세션 정보]
    - 작업 분류: {session.task_id}
    - 목표 시간: {session.goal_duration or 0}분
    - 실제 소요 시간: {round((session.duration or 0) / 60, 1)}분
    - 방해/개입 횟수: {session.interruption_count}회

    [활동 상세 로그]
    {event_context}

    [작성 가이드라인]
    1. summary_title: 사용자의 집중 패턴을 한 단어로 정의하세요 (예: '초집중 모드', '멀티태스킹형').
    2. summary_description: 로그를 기반으로 무엇을 잘했고, 무엇이 방해되었는지 구체적으로 서술하세요.
    3. focus_stats: 로그의 타임스탬프 간격을 분석하여 최대 연속 집중 시간을 추정하세요.
    4. distraction_ratio: 작업과 관련 없는 앱(SNS, 유튜브 등)의 비중을 계산하세요.
    """

    try:
        # 4. Gemini API 호출 (Response Schema 강제)
        response = client.models.generate_content(
            model="gemini-2.0-flash", 
            contents=prompt,
            config={
                "response_mime_type": "application/json",
                "response_schema": InsightResponse
            }
        )
        
        # 5. 파싱된 결과 반환
        return response.parsed

    except Exception as e:
        print(f"LLM Analysis Error: {str(e)}")
        raise HTTPException(
            status_code=500, 
            detail=f"AI 분석 중 오류가 발생했습니다: {str(e)}"
        )

@router.get("/last-session", response_model=InsightResponse)
async def analyze_last_session(user_id: str = Depends(get_current_user_id)):
    """
    사용자의 가장 최근 종료된 세션을 찾아 분석합니다. (메인 대시보드용)
    """
    sessions = await session_crud.get_sessions(user_id, limit=1)
    if not sessions:
        raise HTTPException(status_code=404, detail="분석할 최근 세션이 없습니다.")
    
    return await analyze_session_insight(sessions[0].id, user_id)