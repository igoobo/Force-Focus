# backend/app/api/endpoints/health.py

from fastapi import APIRouter

from app.db.mongo import get_db

router = APIRouter(tags=["Health"])


@router.get("/health")
async def health_check():
    """
    [운영] 헬스 체크
    - 서버 생존 여부 + Mongo(Atlas) 연결 여부를 빠르게 확인하기 위한 엔드포인트
    - GCP/로드밸런서/모니터링에서 주로 사용
    """
    mongo_ok = False
    mongo_error = None

    try:
        # MongoDB ping: 연결/권한/네트워크 문제를 가장 단순하게 확인
        await get_db().command("ping")
        mongo_ok = True
    except Exception as e:
        mongo_error = str(e)

    return {
        "status": "ok" if mongo_ok else "degraded",
        "mongo": mongo_ok,
        # 운영에서는 숨겨도 되는데, 지금은 디버깅 편의를 위해 포함
        "mongo_error": mongo_error,
    }
