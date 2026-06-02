import os
from typing import List


class Settings:
    """Конфигурация приложения"""

    # Сервер
    HOST: str = os.getenv("HOST", "0.0.0.0")
    PORT: int = int(os.getenv("PORT", "8001"))
    RELOAD: bool = os.getenv("RELOAD", "False").lower() == "true"

    # Директории
    UPLOAD_DIR: str = os.getenv("UPLOAD_DIR", "uploads")
    RESULTS_DIR: str = os.getenv("RESULTS_DIR", "results")

    # CORS
    CORS_ALLOW_ORIGINS: List[str] = os.getenv("CORS_ALLOW_ORIGINS", "*").split(",")
    CORS_ALLOW_CREDENTIALS: bool = True
    CORS_ALLOW_METHODS: List[str] = ["*"]
    CORS_ALLOW_HEADERS: List[str] = ["*"]

    # IP Whitelist
    ALLOWED_IPS: List[str] = os.getenv("ALLOWED_IPS", "").split(",")

    # ComfyUI
    COMFYUI_URL: str = os.getenv("COMFYUI_URL", "http://127.0.0.1:8188")
    COMFYUI_TIMEOUT: int = int(os.getenv("COMFYUI_TIMEOUT", "5"))

    @classmethod
    def ensure_directories(cls):
        """Создание необходимых директорий"""
        os.makedirs(cls.UPLOAD_DIR, exist_ok=True)
        os.makedirs(cls.RESULTS_DIR, exist_ok=True)


settings = Settings()