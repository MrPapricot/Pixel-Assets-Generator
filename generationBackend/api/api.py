import json
import os
import uuid
import time
from typing import Optional, Dict, Any
from enum import Enum

from fastapi import FastAPI, File, UploadFile, Form, HTTPException, BackgroundTasks
from fastapi.responses import FileResponse
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
import uvicorn

# Импортируем ваш класс
# Предполагаем, что utils.py лежит в той же директории
from utils import PixelArtTransformer

# ============== Модели данных для API ==============

class ColorMode(str, Enum):
    MONOCHROME = "monochrome"
    LIMITED_4 = "limited_4"
    LIMITED_8 = "limited_8"
    LIMITED_16 = "limited_16"
    LIMITED_32 = "limited_32"
    LIMITED_256 = "limited_256"
    TRUE_COLOR = "true_color"

class OutlineType(str, Enum):
    BLACK_OUTLINE = "black_outline"
    COLORED_OUTLINE = "colored_outline"
    NO_OUTLINE = "no_outline"

class StylizationType(str, Enum):
    MINIMALIST = "minimalist"
    CLEAN = "clean"
    DITHERED = "dithered"
    HIGH_DETAIL = "high_detail"

class LightingType(str, Enum):
    FLAT = "flat"
    CELL_SHADED = "cell_shaded"
    SMOOTH = "smooth"

class ProjectionType(str, Enum):
    SIDE_VIEW = "side_view"
    TOP_DOWN = "top_down"
    ISOMETRIC_2_1 = "isometric_2_1"
    ISOMETRIC_1_1 = "isometric_1_1"

class AssetType(str, Enum):
    CHARACTER = "character"
    PROP = "prop"
    TILE = "tile"

class BaseResolution(str, Enum):
    R16x16 = "16x16"
    R24x24 = "24x24"
    R32x32 = "32x32"
    R48x48 = "48x48"
    R64x64 = "64x64"
    R128x128 = "128x128"

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

class TransformResponse(BaseModel):
    task_id: str = Field(..., description="ID задачи")
    status: str = Field(..., description="Статус обработки")
    message: str = Field(..., description="Сообщение")

class TaskStatus(BaseModel):
    task_id: str
    status: str  # 'pending', 'processing', 'completed', 'failed'
    progress: int = Field(0, ge=0, le=100)
    created_at: float
    updated_at: float
    result_url: Optional[str] = None
    error: Optional[str] = None
    config: Optional[Dict[str, Any]] = None

# ============== Инициализация FastAPI ==============

app = FastAPI(
    title="Pixel Art Transformer API",
    description="API для преобразования изображений в пиксель-арт с использованием AI",
    version="1.0.0"
)

# Добавляем CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Хранилище задач (в production лучше использовать Redis)
tasks_store: Dict[str, TaskStatus] = {}

# Директории для хранения файлов
UPLOAD_DIR = "uploads"
RESULTS_DIR = "results"

os.makedirs(UPLOAD_DIR, exist_ok=True)
os.makedirs(RESULTS_DIR, exist_ok=True)

# ============== Вспомогательные функции ==============

def process_image_task(task_id: str, image_path: str, config: Dict[str, Any]):
    """Фоновая задача обработки изображения"""
    try:
        # Обновляем статус
        tasks_store[task_id].status = "processing"
        tasks_store[task_id].progress = 10
        tasks_store[task_id].updated_at = time.time()
        
        # Инициализируем трансформер
        tasks_store[task_id].progress = 20
        transformer = PixelArtTransformer(config)
        
        # Путь для результата
        result_filename = f"{task_id}.png"
        result_path = os.path.join(RESULTS_DIR, result_filename)
        
        tasks_store[task_id].progress = 30
        
        # Обрабатываем изображение
        success = transformer.process_image(image_path, result_path)
        
        if success:
            tasks_store[task_id].status = "completed"
            tasks_store[task_id].progress = 100
            tasks_store[task_id].result_url = f"/api/v1/download/{task_id}"
        else:
            tasks_store[task_id].status = "failed"
            tasks_store[task_id].error = "Ошибка при обработке изображения"
            
    except Exception as e:
        tasks_store[task_id].status = "failed"
        tasks_store[task_id].error = str(e)
    
    finally:
        tasks_store[task_id].updated_at = time.time()
        
        # Очищаем временный файл
        if os.path.exists(image_path):
            os.remove(image_path)

# ============== API Эндпоинты ==============

@app.get("/")
async def root():
    """Корневой эндпоинт"""
    return {
        "name": "Pixel Art Transformer API",
        "version": "1.0.0",
        "docs": "/docs",
        "redoc": "/redoc"
    }

@app.get("/health")
async def health_check():
    """Проверка здоровья сервиса"""
    return {
        "status": "healthy",
        "comfyui_available": check_comfyui_available()
    }

def check_comfyui_available() -> bool:
    """Проверка доступности ComfyUI"""
    try:
        import requests
        response = requests.get("http://127.0.0.1:8188/system_stats", timeout=5)
        return response.status_code == 200
    except:
        return False

@app.post("/api/v1/transform/file", response_model=TransformResponse)
async def transform_from_file(
    background_tasks: BackgroundTasks,
    file: UploadFile = File(..., description="Изображение для обработки"),
    config: Optional[str] = Form(None, description="JSON конфигурация")
):
    """
    Обработка изображения из загруженного файла
    
    - **file**: Файл изображения (PNG, JPG, JPEG)
    - **config**: Опциональная JSON конфигурация
    """
    # Проверяем тип файла
    if not file.content_type.startswith('image/'):
        raise HTTPException(400, "Файл должен быть изображением")
    
    # Генерируем ID задачи
    task_id = str(uuid.uuid4())
    
    # Сохраняем загруженный файл
    file_ext = os.path.splitext(file.filename)[1] or ".png"
    upload_path = os.path.join(UPLOAD_DIR, f"{task_id}{file_ext}")
    
    content = await file.read()
    with open(upload_path, "wb") as f:
        f.write(content)
    
    # Парсим конфигурацию
    if config:
        try:
            config_dict = json.loads(config)
        except json.JSONDecodeError:
            os.remove(upload_path)
            raise HTTPException(400, "Невалидный JSON в конфигурации")
    else:
        # Конфигурация по умолчанию
        config_dict = TransformConfig().dict()
    
    # Создаем запись о задаче
    tasks_store[task_id] = TaskStatus(
        task_id=task_id,
        status="pending",
        progress=0,
        created_at=time.time(),
        updated_at=time.time(),
        config=config_dict
    )
    
    # Запускаем фоновую обработку
    background_tasks.add_task(process_image_task, task_id, upload_path, config_dict)
    
    return TransformResponse(
        task_id=task_id,
        status="pending",
        message="Задача добавлена в очередь"
    )

@app.get("/api/v1/status/{task_id}")
async def get_task_status(task_id: str):
    """
    Получение статуса задачи по ID
    
    - **task_id**: ID задачи
    """
    if task_id not in tasks_store:
        raise HTTPException(404, "Задача не найдена")
    
    return tasks_store[task_id]

@app.get("/api/v1/download/{task_id}")
async def download_result(task_id: str):
    """
    Скачивание результата обработки
    
    - **task_id**: ID задачи
    """
    if task_id not in tasks_store:
        raise HTTPException(404, "Задача не найдена")
    
    task = tasks_store[task_id]
    
    if task.status != "completed":
        raise HTTPException(400, f"Задача не завершена. Статус: {task.status}")
    
    result_path = os.path.join(RESULTS_DIR, f"{task_id}.png")
    
    if not os.path.exists(result_path):
        raise HTTPException(404, "Файл результата не найден")
    
    return FileResponse(
        result_path,
        media_type="image/png",
        filename=f"pixel_art_{task_id}.png"
    )

@app.get("/api/v1/tasks")
async def list_tasks(limit: int = 50, status: Optional[str] = None):
    """
    Получение списка задач
    
    - **limit**: Максимальное количество задач
    - **status**: Фильтр по статусу (pending, processing, completed, failed)
    """
    tasks = list(tasks_store.values())
    
    if status:
        tasks = [t for t in tasks if t.status == status]
    
    # Сортируем по времени создания (новые сначала)
    tasks.sort(key=lambda x: x.created_at, reverse=True)
    
    return {
        "total": len(tasks),
        "tasks": tasks[:limit]
    }

@app.delete("/api/v1/tasks/{task_id}")
async def delete_task(task_id: str):
    """
    Удаление задачи и связанных файлов
    
    - **task_id**: ID задачи
    """
    if task_id not in tasks_store:
        raise HTTPException(404, "Задача не найдена")
    
    # Удаляем файлы
    result_path = os.path.join(RESULTS_DIR, f"{task_id}.png")
    if os.path.exists(result_path):
        os.remove(result_path)
    
    # Удаляем из хранилища
    del tasks_store[task_id]
    
    return {"message": f"Задача {task_id} удалена"}

@app.get("/api/v1/config/presets")
async def get_config_presets():
    """
    Получение предустановленных конфигураций
    """
    presets = {
        "pixel_character": {
            "technical": {
                "base_resolution": "32x32",
                "color_mode": "limited_16",
                "outline": "black_outline",
                "width": 8,
                "height": 8
            },
            "visual": {
                "stylization": "clean",
                "lighting": "cell_shaded",
                "projection": "side_view"
            },
            "functional": {
                "asset_type": "character"
            }
        },
        "isometric_prop": {
            "technical": {
                "base_resolution": "64x64",
                "color_mode": "limited_32",
                "outline": "colored_outline",
                "width": 16,
                "height": 16
            },
            "visual": {
                "stylization": "high_detail",
                "lighting": "smooth",
                "projection": "isometric_2_1"
            },
            "functional": {
                "asset_type": "prop"
            }
        },
        "minimalist_tile": {
            "technical": {
                "base_resolution": "16x16",
                "color_mode": "limited_8",
                "outline": "no_outline",
                "width": 4,
                "height": 4
            },
            "visual": {
                "stylization": "minimalist",
                "lighting": "flat",
                "projection": "top_down"
            },
            "functional": {
                "asset_type": "tile"
            }
        }
    }
    
    return presets

# ============== Запуск приложения ==============

if __name__ == "__main__":
    uvicorn.run(
        "api:app",
        host="127.0.0.1",
        port=8001,
        reload=False,
        log_level="info"
    )