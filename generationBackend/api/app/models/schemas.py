from typing import Optional, Dict, Any
from pydantic import BaseModel, Field
from app.models.enums import (
    ColorMode, OutlineType, StylizationType,
    LightingType, ProjectionType, AssetType, BaseResolution
)


class TechnicalConfig(BaseModel):
    base_resolution: BaseResolution = Field(BaseResolution.R64x64, description="Базовое разрешение")
    color_mode: ColorMode = Field(ColorMode.LIMITED_16, description="Цветовой режим")
    outline: OutlineType = Field(OutlineType.BLACK_OUTLINE, description="Тип контура")
    width: int = Field(8, ge=1, le=64, description="Количество тайлов по ширине")
    height: int = Field(8, ge=1, le=64, description="Количество тайлов по высоте")


class VisualConfig(BaseModel):
    stylization: StylizationType = Field(StylizationType.CLEAN, description="Тип стилизации")
    lighting: LightingType = Field(LightingType.FLAT, description="Тип освещения")
    projection: ProjectionType = Field(ProjectionType.SIDE_VIEW, description="Тип проекции")


class FunctionalConfig(BaseModel):
    asset_type: AssetType = Field(AssetType.PROP, description="Тип ассета")


class ComfyUIConfig(BaseModel):
    url: str = Field("http://127.0.0.1:8188", description="URL ComfyUI")
    use_lightning_lora: bool = Field(True, description="Использовать Lightning LoRA")
    random_seed: bool = Field(True, description="Случайный seed")
    timeout: int = Field(180, ge=30, le=600, description="Таймаут в секундах")


class TransformConfig(BaseModel):
    technical: TechnicalConfig = Field(default_factory=TechnicalConfig)
    visual: VisualConfig = Field(default_factory=VisualConfig)
    functional: FunctionalConfig = Field(default_factory=FunctionalConfig)
    comfyui: Optional[ComfyUIConfig] = Field(None, description="Настройки ComfyUI")

    class Config:
        json_schema_extra = {
            "example": {
                "technical": {
                    "base_resolution": "64x64",
                    "color_mode": "limited_16"
                }
            }
        }


class TransformResponse(BaseModel):
    task_id: str = Field(..., description="ID задачи")
    status: str = Field(..., description="Статус обработки")
    message: str = Field(..., description="Сообщение")