from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from app.config import settings
from app.middleware.ip_whitelist import IPWhitelistMiddleware
from app.api.routes import health, transform, tasks, config

# Создаем приложение
app = FastAPI(
    title="Pixel Art Transformer API",
    description="API для преобразования изображений в пиксель-арт с использованием AI",
    version="1.0.0"
)

# Добавляем CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.CORS_ALLOW_ORIGINS,
    allow_credentials=settings.CORS_ALLOW_CREDENTIALS,
    allow_methods=settings.CORS_ALLOW_METHODS,
    allow_headers=settings.CORS_ALLOW_HEADERS,
)

# Добавляем IP Whitelist middleware
app.add_middleware(IPWhitelistMiddleware)

# Регистрируем роутеры
app.include_router(health.router)
app.include_router(transform.router)
app.include_router(tasks.router)
app.include_router(config.router)

# Создаем директории при старте
settings.ensure_directories()