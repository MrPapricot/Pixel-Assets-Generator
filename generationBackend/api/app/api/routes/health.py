from fastapi import APIRouter
import requests
from app.config import settings

router = APIRouter(tags=["Health"])

def check_comfyui_available() -> bool:
    """Проверка доступности ComfyUI"""
    try:
        response = requests.get(f"{settings.COMFYUI_URL}/system_stats", timeout=settings.COMFYUI_TIMEOUT)
        return response.status_code == 200
    except:
        return False

@router.get("/")
async def root():
    """Корневой эндпоинт"""
    return {
        "name": "Pixel Art Transformer API",
        "version": "1.0.0",
        "docs": "/docs",
        "redoc": "/redoc"
    }

@router.get("/health")
async def health_check():
    """Проверка здоровья сервиса"""
    return {
        "status": "healthy",
        "comfyui_available": check_comfyui_available()
    }