from typing import Optional, Dict, Any
from pydantic import BaseModel, Field
from app.models.enums import TaskStatusEnum

class TaskStatus(BaseModel):
    task_id: str
    status: TaskStatusEnum
    progress: int = Field(0, ge=0, le=100)
    created_at: float
    updated_at: float
    result_url: Optional[str] = None
    error: Optional[str] = None
    config: Optional[Dict[str, Any]] = None