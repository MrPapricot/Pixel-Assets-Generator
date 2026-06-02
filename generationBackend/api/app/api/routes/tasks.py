from typing import Optional
from fastapi import APIRouter, HTTPException, Depends
from fastapi.responses import FileResponse
from app.services.task_manager import task_manager
from app.api.dependencies import get_file_service, get_task_manager
from app.models.enums import TaskStatusEnum

router = APIRouter(prefix="/api/v1", tags=["Tasks"])


@router.get("/status/{task_id}")
async def get_task_status(task_id: str, task_mgr=Depends(get_task_manager)):
    """Получение статуса задачи по ID"""
    task = task_mgr.get_task(task_id)
    if not task:
        raise HTTPException(404, "Задача не найдена")
    return task


@router.get("/download/{task_id}")
async def download_result(task_id: str, task_mgr=Depends(get_task_manager), file_svc=Depends(get_file_service)):
    """Скачивание результата обработки"""
    task = task_mgr.get_task(task_id)
    if not task:
        raise HTTPException(404, "Задача не найдена")

    if task.status != TaskStatusEnum.COMPLETED:
        raise HTTPException(400, f"Задача не завершена. Статус: {task.status}")

    result_path = file_svc.get_result_path(task_id)
    if not file_svc.result_exists(task_id):
        raise HTTPException(404, "Файл результата не найден")

    return FileResponse(
        result_path,
        media_type="image/png",
        filename=f"pixel_art_{task_id}.png"
    )


@router.get("/tasks")
async def list_tasks(limit: int = 50, status: Optional[str] = None, task_mgr=Depends(get_task_manager)):
    """Получение списка задач"""
    tasks = task_mgr.list_tasks(limit, status)
    return {
        "total": len(tasks),
        "tasks": tasks
    }


@router.delete("/tasks/{task_id}")
async def delete_task(task_id: str, task_mgr=Depends(get_task_manager), file_svc=Depends(get_file_service)):
    """Удаление задачи и связанных файлов"""
    if not task_mgr.get_task(task_id):
        raise HTTPException(404, "Задача не найдена")

    # Удаляем файлы
    result_path = file_svc.get_result_path(task_id)
    file_svc.delete_file(result_path)

    # Удаляем из хранилища
    task_mgr.delete_task(task_id)

    return {"message": f"Задача {task_id} удалена"}