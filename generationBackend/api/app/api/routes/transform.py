from fastapi import APIRouter, File, UploadFile, Form, HTTPException, BackgroundTasks, Depends
from app.models.schemas import TransformResponse, TransformConfig
from app.services.task_manager import task_manager
from app.services.transformer import TransformerService
from app.api.dependencies import get_file_service, get_task_manager
from app.models.enums import TaskStatusEnum

router = APIRouter(prefix="/api/v1/transform", tags=["Transform"])


@router.post("/file", response_model=TransformResponse)
async def transform_from_file(
        background_tasks: BackgroundTasks,
        file: UploadFile = File(..., description="Изображение для обработки"),
        config: str = Form(None, description="JSON конфигурация"),
        task_mgr=Depends(get_task_manager),
        file_svc=Depends(get_file_service)
):
    """
    Обработка изображения из загруженного файла
    """
    # Парсим конфигурацию
    config_dict = file_svc.parse_config(config)
    if not config_dict:
        config_dict = TransformConfig().dict()

    # Создаем задачу
    task_id = task_mgr.create_task(config_dict)

    # Сохраняем файл
    upload_path = await file_svc.save_uploaded_file(task_id, file)

    # Запускаем фоновую обработку
    background_tasks.add_task(
        process_image_background,
        task_id, upload_path, config_dict, task_mgr, file_svc
    )

    return TransformResponse(
        task_id=task_id,
        status=TaskStatusEnum.PENDING,
        message="Задача добавлена в очередь"
    )


def process_image_background(task_id: str, image_path: str, config: dict, task_mgr, file_svc):
    """Фоновая обработка изображения"""
    try:
        # Обновляем статус
        task_mgr.update_task_status(task_id, TaskStatusEnum.PROCESSING, progress=10)

        # Инициализируем трансформер
        task_mgr.update_task_status(task_id, TaskStatusEnum.PROCESSING, progress=20)

        # Обрабатываем изображение
        result_path = file_svc.get_result_path(task_id)
        task_mgr.update_task_status(task_id, TaskStatusEnum.PROCESSING, progress=30)

        success = TransformerService.process_image(config, image_path, result_path)

        if success:
            task_mgr.update_task_status(task_id, TaskStatusEnum.COMPLETED, progress=100)
            task_mgr.update_task_result(task_id, f"/api/v1/download/{task_id}")
        else:
            task_mgr.update_task_status(
                task_id, TaskStatusEnum.FAILED,
                error="Ошибка при обработке изображения"
            )

    except Exception as e:
        task_mgr.update_task_status(task_id, TaskStatusEnum.FAILED, error=str(e))

    finally:
        # Очищаем временный файл
        file_svc.delete_file(image_path)