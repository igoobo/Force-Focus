from pydantic import BaseModel, Field
from typing import List, Optional

class DetailCard(BaseModel):
    title: str = Field(description="섹션 제목 (예: 활동 요약, 긍정적 평가 등)")
    items: List[str] = Field(description="글머리 기호로 표시될 분석 내용 리스트")

class FocusStats(BaseModel):
    max_continuous: str = Field(description="최대 연속 몰입 시간 (예: '35분')")
    threshold: str = Field(description="인지적 임계점 (예: '32분')")
    average_score: str = Field(description="평균 집중도 (예: '77.5%')")

class InsightResponse(BaseModel):
    # 1. 종합 분석 섹션 데이터
    summary_title: str = Field(description="사용자 유형 키워드 (예: '효율적 사용자')")
    summary_badge: str = Field(description="종합 분석 배지 (예: 'Success Profile')")
    summary_description: str = Field(description="상세 분석 리포트 전문")
    summary_cards: List[DetailCard] = Field(description="활동 요약/긍정 평가/개선점 카드 데이터")

    # 2. 집중도 섹션 데이터
    focus_badge: str = Field(description="집중도 섹션 배지 (예: 'Cognitive Analysis')")
    focus_stats: FocusStats
    focus_insight_title: str = Field(description="심층 분석 리포트 제목")
    focus_insight_content: str = Field(description="인지적 부하 및 흐름 분석 내용")

    # 3. 피로도 섹션 데이터
    fatigue_badge: str = Field(description="피로도 섹션 배지 (예: 'Fatigue Management')")
    fatigue_description: str = Field(description="디지털 피로도 분석 내용")
    distraction_ratio: float = Field(description="방해 요소 점유율 (0~100 사이 실수)")
    distraction_app: str = Field(description="주요 방해 앱 이름 (예: 'Discord')")
    recovery_strategies: List[DetailCard] = Field(description="회복 전략 아이템")