import os
from typing import Any, Dict
from pathlib import Path
from fastapi import APIRouter, Depends, HTTPException, status
from fastapi.responses import FileResponse

from app.api import deps
# User 모델 의존성 제거 (deps.get_current_user_id 사용 권장) - 여기선 폴더명으로만 쓰므로 id만 있으면 됨

router = APIRouter()

# ---------------------------------------------------------
# 경로 설정 (train.py와 동일한 규칙 적용)
# ---------------------------------------------------------
# 현재 파일: backend/app/api/endpoints/desktop/models.py
# Root: backend/
BASE_DIR = Path(__file__).resolve().parents[4] 
MODEL_STORAGE_DIR = BASE_DIR / "storage" / "models"

@router.get("/latest", response_model=Dict[str, Any])
def check_latest_model_version(
    user_id: str = Depends(deps.get_current_user_id),
) -> Any:
    """
    [Desktop Agent] 최신 모델 버전 확인
    
    Returns:
        {
            "version": "20240210123456",
            "download_urls": {
                "model": "/api/v1/desktop/models/20240210123456/model.onnx",
                "scaler": "/api/v1/desktop/models/20240210123456/scaler_params.json"
            }
        }
    """
    user_model_dir = MODEL_STORAGE_DIR / user_id

    # 1. 유저 폴더가 없으면 404 (아직 학습된 모델 없음)
    if not user_model_dir.exists():
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="No models trained yet."
        )

    # 2. 버전 디렉토리 검색 (디렉토리만 필터링)
    # 버전명은 YYYYMMDDHHMMSS 포맷이므로 문자열 정렬 시 최신순 보장
    versions = sorted(
        [d.name for d in user_model_dir.iterdir() if d.is_dir()],
        reverse=True
    )

    if not versions:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Model directory exists but no versions found."
        )

    latest_version = versions[0]
    
    # 3. 파일 존재 여부 최종 확인
    latest_dir = user_model_dir / latest_version
    model_path = latest_dir / "model.onnx"
    scaler_path = latest_dir / "scaler_params.json"

    if not model_path.exists() or not scaler_path.exists():
        # 최신 폴더에 파일이 없으면 그 다음 버전을 찾거나 에러 처리 (여기선 에러)
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="Latest model files are corrupted or missing."
        )

    # 4. 메타데이터 반환
    base_url = f"/api/v1/desktop/models/{latest_version}"
    return {
        "status": "success",
        "version": latest_version,
        "download_urls": {
            "model": f"{base_url}/model.onnx",
            "scaler": f"{base_url}/scaler_params.json"
        }
    }

@router.get("/{version}/{filename}", response_class=FileResponse)
def download_model_file(
    version: str,
    filename: str,
    user_id: str = Depends(deps.get_current_user_id),
) -> Any:
    """
    [Desktop Agent] 특정 버전의 모델 파일 다운로드
    """
    # 보안: 파일명 필터링 (Directory Traversal 방지)
    if filename not in ["model.onnx", "scaler_params.json"]:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Invalid filename request."
        )

    # 경로 구성
    file_path = MODEL_STORAGE_DIR / user_id / version / filename

    if not file_path.exists():
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="File not found."
        )

    return FileResponse(
        path=file_path,
        filename=filename,
        media_type="application/octet-stream"
    )