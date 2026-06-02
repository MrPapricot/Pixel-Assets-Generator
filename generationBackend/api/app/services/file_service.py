import os
import json
from typing import Optional
from fastapi import HTTPException, UploadFile


class FileService:
    """Сервис для работы с файлами"""

    def __init__(self, upload_dir: str, results_dir: str):
        self.upload_dir = upload_dir
        self.results_dir = results_dir

    async def save_uploaded_file(self, task_id: str, file: UploadFile) -> str:
        """Сохранение загруженного файла"""
        if not file.content_type.startswith('image/'):
            raise HTTPException(400, "Файл должен быть изображением")

        file_ext = os.path.splitext(file.filename)[1] or ".png"
        upload_path = os.path.join(self.upload_dir, f"{task_id}{file_ext}")

        content = await file.read()
        with open(upload_path, "wb") as f:
            f.write(content)

        return upload_path

    def delete_file(self, file_path: str):
        """Удаление файла"""
        if os.path.exists(file_path):
            os.remove(file_path)

    def get_result_path(self, task_id: str) -> str:
        """Получение пути к результату"""
        return os.path.join(self.results_dir, f"{task_id}.png")

    def result_exists(self, task_id: str) -> bool:
        """Проверка существования результата"""
        return os.path.exists(self.get_result_path(task_id))

    def parse_config(self, config_json: Optional[str]) -> dict:
        """Парсинг JSON конфигурации"""
        if config_json:
            try:
                return json.loads(config_json)
            except json.JSONDecodeError:
                raise HTTPException(400, "Невалидный JSON в конфигурации")
        return None