import os
from typing import Any
from fastapi import APIRouter, Depends, HTTPException, status
from fastapi.responses import FileResponse

from app.api import deps
from app.core.config import settings
from app.models.user import User

router = APIRouter()

# 모델 저장소 경로
BASE_MODEL_DIR = settings.MODEL_STORAGE_DIR

@router.get("/latest", response_class=FileResponse)
def download_latest_model(
    current_user: User = Depends(deps.get_current_user),
) -> Any:
    """
    [Desktop Agent] 개인화 모델 다운로드
    
    1. 요청한 유저(current_user)의 ID 기반 폴더를 탐색합니다.
    2. 가장 최신 ONNX 파일을 찾아 스트리밍 전송합니다.
    """
    
    # 1. 유저 전용 경로 구성 (예: models_storage/65a1b2...)
    user_model_dir = os.path.join(BASE_MODEL_DIR, str(current_user.id))

    # 2. 유효성 검사: 폴더가 없거나 파일이 없는 경우
    if not os.path.exists(user_model_dir):
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Personalized model directory not found."
        )

    # 3. .onnx 파일 검색
    files = [
        os.path.join(user_model_dir, f) 
        for f in os.listdir(user_model_dir) 
        if f.endswith(".onnx")
    ]

    if not files:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="No .onnx model files found for this user."
        )

    # 4. 최신 파일 선정 (OS 파일 수정 시간 기준)
    latest_file = max(files, key=os.path.getmtime)
    filename = os.path.basename(latest_file)

    # 5. 파일 전송 (MIME Type: application/octet-stream)
    return FileResponse(
        path=latest_file,
        filename=filename,
        media_type="application/octet-stream"
    )