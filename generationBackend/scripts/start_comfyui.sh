#!/bin/bash

# Переходим в директорию ComfyUI
cd /workspace/ComfyUI

# Формируем аргументы
ARGS="--listen 0.0.0.0 --port ${COMFYUI_PORT:-8188}"

# Добавляем CPU если нужно
if [ "$USE_CPU" = "true" ]; then
    echo "🖥️  Запуск на CPU..."
    ARGS="$ARGS --cpu"
fi

# Загрузка моделей (опционально)
if [ "$DOWNLOAD_MODELS" = "true" ] && [ -f "/scripts/download_models.py" ]; then
    echo "📥 Загрузка моделей..."
    python /scripts/download_models.py
fi

echo "🚀 Запуск ComfyUI: $ARGS"
exec python main.py $ARGS