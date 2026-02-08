# backend/app/api/endpoints/web/insight.py

import os
from typing import Optional
from fastapi import APIRouter, HTTPException, Depends, Query
from google import genai
from datetime import datetime, timezone

from app.api.deps import get_current_user_id
from app.crud import sessions as session_crud
from app.crud import events as event_crud
from app.schemas.insight import InsightResponse

router = APIRouter(prefix="/insight", tags=["AI Insight"])

# Gemini 클라이언트 설정
client = genai.Client(api_key=os.getenv("GEMINI_API_KEY"))

@router.get("/analyze/{session_id}", response_model=InsightResponse)
async def analyze_session_insight(
    session_id: str,
    user_id: str = Depends(get_current_user_id)
):
    """
    세션 데이터를 분석하여 리포트를 생성합니다. 
    데이터 부족 시 영문 배지와 구조화된 범용 피드백을 제공합니다.
    """
    # 1. 데이터 통합 컨텍스트 및 세션 정보 추출
    event_context = await session_crud.get_session_full_context(user_id, session_id)
    session = await session_crud.get_session(session_id)
    if not session:
         sessions = await session_crud.get_sessions(user_id, limit=1)
         session = sessions[0] if sessions else None

    # 데이터 부족 여부 판별
    is_data_insufficient = "기록된 활동 로그가 없습니다" in event_context or not session

    # 2. 가변 프롬프트 구성
    if is_data_insufficient:
        prompt = """
        사용자의 작업 데이터가 부족합니다. 전문 생산성 코치로서 '범용 생산성 최적화 가이드'를 InsightResponse 형식으로 작성하세요.
        프론트엔드 UI의 모든 섹션이 채워지도록 상세하고 풍성한 내용을 생성해야 합니다.

        [스타일 및 구조 지침]
        1. 영문 배지 적용: 
           - summary_badge: 'READY'
           - focus_badge: 'STANDBY'
           - fatigue_badge: 'STABLE'
        2. 카드 제목 (이모티콘 포함) 및 구성 (summary_cards):
           - 첫 번째 카드: 제목 '📝 요약', 데이터가 적어 기본 분석 모드로 동작 중임을 알리고 딥워크의 중요성 설명.
           - 두 번째 카드: 제목 '💡 추천 실천 사항', 뽀모도로 기법(25분 집중/5분 휴식) 등 데이터가 없을 때 추천하는 습관 제안.
           - 세 번째 카드: 제목 '⚠️ 주의 사항', 멀티태스킹 방지 및 알림 관리 등 주의할 점 제안.
        3. 텍스트 강조: 중요 키워드는 반드시 **볼드체**(**내용**)를 사용하세요.
        4. 기타 필수 필드: 
           - focus_stats: max_continuous '25분(권장)', threshold '양호', average_score '70'.
           - focus_insight_title: '뇌과학 기반 집중력 향상법'.
           - focus_insight_content: 도파민 관리 및 환경 설정법 상세 서술.
           - fatigue_description: 디지털 피로도 예방을 위한 20-20-20 규칙 등 상세 서술.
           - recovery_strategies: '안구 건조 예방', '전신 스트레칭' 등 2개 이상의 전략을 상세 항목과 함께 제공.
        5. 모든 응답은 InsightResponse JSON 구조를 엄격히 따를 것.
        """
    else:
        prompt = f"""
        당신은 전문 생산성 분석가입니다. 아래 데이터를 분석하여 InsightResponse 형식으로 응답하세요.
        
        [데이터]
        - 작업: {session.task_id if session else '미분류'}
        - 실제 로그: {event_context}

        [분석 지침]
        1. 영문 배지 적용: 성과에 따라 'EXCELLENT', 'DEEP FOCUS', 'WARNING' 등 영문 대문자 사용.
        2. 카드 제목 (이모티콘 포함):
           - 첫 번째 카드: '📊 요약'
           - 두 번째 카드: '✅ 양호한 점'
           - 세 번째 카드: '🚀 개선이 필요한 점'
        3. 텍스트 강조: 앱 이름, 특정 시간대, 집중도 등 핵심 요소는 반드시 **볼드체**(**내용**)를 사용하세요.
        4. 모든 응답은 InsightResponse JSON 구조를 엄격히 따를 것.
        """

    try:
        # 3. Gemini API 호출
        response = client.models.generate_content(
            model="gemini-2.0-flash", 
            contents=prompt,
            config={
                "response_mime_type": "application/json",
                "response_schema": InsightResponse
            }
        )
        return response.parsed

    except Exception as e:
        print(f"LLM Analysis Error: {str(e)}")
        raise HTTPException(status_code=500, detail="AI 분석 중 오류가 발생했습니다.")

@router.get("/last-session", response_model=InsightResponse)
async def analyze_last_session(user_id: str = Depends(get_current_user_id)):
    """
    최근 세션을 분석하거나 데이터가 없으면 가이드 모드 결과를 반환합니다.
    """
    sessions = await session_crud.get_sessions(user_id, limit=1)
    target_id = sessions[0].id if sessions else "no_data"
    return await analyze_session_insight(target_id, user_id)