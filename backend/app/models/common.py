# backend/app/models/common.py

from typing import Annotated, Any
from bson import ObjectId
from pydantic import PlainSerializer, BeforeValidator


def _to_object_id(v: Any) -> ObjectId:
    """
    DB/요청 등에서 들어오는 _id 값을 ObjectId로 정규화.
    - 이미 ObjectId면 그대로 반환
    - 24 hex string이면 ObjectId로 변환
    - 그 외는 에러
    """
    if isinstance(v, ObjectId):
        return v
    if isinstance(v, str) and ObjectId.is_valid(v):
        return ObjectId(v)
    raise TypeError("Invalid ObjectId")


PyObjectId = Annotated[
    ObjectId,
    BeforeValidator(_to_object_id),
    PlainSerializer(lambda x: str(x), return_type=str),
]