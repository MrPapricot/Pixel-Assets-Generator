#!/usr/bin/env python3
"""
Скрипт для загрузки моделей .safetensors для ComfyUI
Поддерживает разные структуры директорий
"""

import os
import sys
import requests
from pathlib import Path
import time

# Определяем базовую директорию в зависимости от образа
def get_base_dir():
    if os.path.exists("/workspace/ComfyUI"):
        return Path("/workspace/ComfyUI/models")
    elif os.path.exists("/comfyui"):
        return Path("/comfyui/models")
    else:
        return Path("/models")

BASE_DIR = get_base_dir()
DOWNLOAD_MODELS = os.getenv("DOWNLOAD_MODELS", "false").lower() == "true"

# Конфигурация моделей
MODELS = {
    "diffusion_models": {
        "filename": "qwen_image_edit_2509_fp8_e4m3fn.safetensors",
        "url": "https://huggingface.co/Comfy-Org/Qwen-Image-Edit_ComfyUI/resolve/main/split_files/diffusion_models/qwen_image_edit_2509_fp8_e4m3fn.safetensors"
    },
    "text_encoders": {
        "filename": "qwen_2.5_vl_7b_fp8_scaled.safetensors",
        "url": "https://huggingface.co/Comfy-Org/Qwen-Image_ComfyUI/resolve/main/split_files/text_encoders/qwen_2.5_vl_7b_fp8_scaled.safetensors"
    },
    "vae": {
        "filename": "qwen_image_vae.safetensors",
        "url": "https://huggingface.co/Comfy-Org/Qwen-Image_ComfyUI/resolve/main/split_files/vae/qwen_image_vae.safetensors"
    },
    "loras": {
        "filename": "Qwen-Image-Edit-2509-Lightning-4steps-V1.0-bf16.safetensors",
        "url": "https://huggingface.co/lightx2v/Qwen-Image-Lightning/resolve/main/Qwen-Image-Edit-2509/Qwen-Image-Edit-2509-Lightning-4steps-V1.0-bf16.safetensors"
    }
}

def download_file(url: str, dest_path: Path, retries: int = 3) -> bool:
    """Скачивание файла с прогресс-баром"""
    if dest_path.exists():
        print(f"✅ Файл уже существует: {dest_path.name}")
        return True
    
    print(f"📥 Скачивание: {dest_path.name}")
    
    for attempt in range(retries):
        try:
            response = requests.get(url, stream=True, timeout=300)
            response.raise_for_status()
            
            total_size = int(response.headers.get('content-length', 0))
            downloaded = 0
            
            dest_path.parent.mkdir(parents=True, exist_ok=True)
            
            with open(dest_path, 'wb') as f:
                for chunk in response.iter_content(chunk_size=8192):
                    f.write(chunk)
                    downloaded += len(chunk)
                    if total_size > 0:
                        percent = (downloaded / total_size) * 100
                        print(f"\r   Прогресс: {percent:.1f}% ({downloaded // 1024 // 1024}MB / {total_size // 1024 // 1024}MB)", end='')
            
            print(f"\n✅ Успешно: {dest_path.name}")
            return True
            
        except Exception as e:
            print(f"\n❌ Ошибка (попытка {attempt + 1}/{retries}): {e}")
            if attempt < retries - 1:
                time.sleep(5)
            else:
                return False
    
    return False

def main():
    """Основная функция загрузки моделей"""
    print(f"📁 Директория моделей: {BASE_DIR}")
    print(f"DOWNLOAD_MODELS = {DOWNLOAD_MODELS}")
    
    if not DOWNLOAD_MODELS:
        print("ℹ️  DOWNLOAD_MODELS=false, пропускаем загрузку")
        return 0
    
    print("🚀 Начинаем загрузку моделей...")
    
    success_count = 0
    failed_models = []
    
    for model_type, model_info in MODELS.items():
        print(f"\n📦 Обработка: {model_type}")
        dest_path = BASE_DIR / model_type / model_info["filename"]
        
        if download_file(model_info["url"], dest_path):
            success_count += 1
        else:
            failed_models.append(model_type)
    
    print("\n" + "="*50)
    print(f"📊 Результат: {success_count}/{len(MODELS)} моделей загружено")
    
    if failed_models:
        print(f"❌ Не загружены: {', '.join(failed_models)}")
        return 1
    
    print("✅ Все модели успешно загружены!")
    return 0

if __name__ == "__main__":
    sys.exit(main())