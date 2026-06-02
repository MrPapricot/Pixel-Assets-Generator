import time
import uuid
from typing import Dict, Optional, Any
from app.models.task import TaskStatus
from app.models.enums import TaskStatusEnum


class TaskManager:
    """Управление задачами (Singleton)"""

    _instance = None
    _tasks_store: Dict[str, TaskStatus] = {}

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance

    def create_task(self, config: Dict[str, Any]) -> str:
        """Создание новой задачи"""
        task_id = str(uuid.uuid4())
        now = time.time()

        self._tasks_store[task_id] = TaskStatus(
            task_id=task_id,
            status=TaskStatusEnum.PENDING,
            progress=0,
            created_at=now,
            updated_at=now,
            config=config
        )

        return task_id

    def get_task(self, task_id: str) -> Optional[TaskStatus]:
        """Получение задачи по ID"""
        return self._tasks_store.get(task_id)

    def update_task_status(self, task_id: str, status: TaskStatusEnum, progress: int = None, error: str = None):
        """Обновление статуса задачи"""
        if task_id in self._tasks_store:
            task = self._tasks_store[task_id]
            task.status = status
            task.updated_at = time.time()
            if progress is not None:
                task.progress = progress
            if error is not None:
                task.error = error

    def update_task_result(self, task_id: str, result_url: str):
        """Обновление результата задачи"""
        if task_id in self._tasks_store:
            self._tasks_store[task_id].result_url = result_url
            self._tasks_store[task_id].updated_at = time.time()

    def list_tasks(self, limit: int = 50, status: Optional[str] = None) -> list:
        """Список задач с фильтрацией"""
        tasks = list(self._tasks_store.values())

        if status:
            tasks = [t for t in tasks if t.status == status]

        tasks.sort(key=lambda x: x.created_at, reverse=True)
        return tasks[:limit]

    def delete_task(self, task_id: str) -> bool:
        """Удаление задачи"""
        if task_id in self._tasks_store:
            del self._tasks_store[task_id]
            return True
        return False


# Глобальный экземпляр
task_manager = TaskManager()