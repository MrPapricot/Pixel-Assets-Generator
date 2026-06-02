from app.services.task_manager import task_manager
from app.services.file_service import FileService
from app.config import settings

# Создаем сервисы
file_service = FileService(settings.UPLOAD_DIR, settings.RESULTS_DIR)

def get_task_manager():
    """Dependency для получения менеджера задач"""
    return task_manager

def get_file_service():
    """Dependency для получения файлового сервиса"""
    return file_service