<!-- # Pixel-Assets-Generator
Сервис для генерации пиксельных асетов из фотографии

# Backend deploy
# ComfyUI:

cd comfyui
python3 -m venv comfy_venv
/comfy_venv/bin/activate
pip install -r requirements.txt

Скачать модели:
https://huggingface.co/Comfy-Org/Qwen-Image-Edit_ComfyUI/blob/main/split_files/diffusion_models/qwen_image_edit_2509_fp8_e4m3fn.safetensors
https://huggingface.co/Comfy-Org/Qwen-Image_ComfyUI/blob/main/split_files/text_encoders/qwen_2.5_vl_7b_fp8_scaled.safetensors
https://huggingface.co/Comfy-Org/Qwen-Image_ComfyUI/blob/main/split_files/vae/qwen_image_vae.safetensors
https://huggingface.co/lightx2v/Qwen-Image-Lightning/resolve/main/Qwen-Image-Edit-2509/Qwen-Image-Edit-2509-Lightning-4steps-V1.0-bf16.safetensors
Разместить модели:
ComfyUI/
    models/
    diffusion_models/
        qwen_image_edit_2509_fp8_e4m3fn.safetensors
    loras/
        Qwen-Image-Lightning-4steps-V1.0.safetensors
    vae/
        qwen_image_vae.safetensors
    text_encoders/
        qwen_2.5_vl_7b_fp8_scaled.safetensors

python main.py --listen 127.0.0.1 --port 8188
добавить --cpu, если вы лох

# Pixel-generation-service:
cd api
python3 -m venv venv
/venv/bin/activate
pip install -r requirements.txt
python api.py

 -->
# Pixel-Assets-Generator

Сервис для генерации пиксельных ассетов из фотографий.

## Backend Deploy

### 1. Развертывание ComfyUI

**Установка:**
```bash
cd comfyui
python3 -m venv comfy_venv
source comfy_venv/bin/activate
pip install -r requirements.txt
```

**Загрузка моделей:**

Скачайте следующие файлы и разместите их согласно структуре ниже:

https://huggingface.co/Comfy-Org/Qwen-Image-Edit_ComfyUI/blob/main/split_files/diffusion_models/qwen_image_edit_2509_fp8_e4m3fn.safetensors
https://huggingface.co/Comfy-Org/Qwen-Image_ComfyUI/blob/main/split_files/text_encoders/qwen_2.5_vl_7b_fp8_scaled.safetensors
https://huggingface.co/Comfy-Org/Qwen-Image_ComfyUI/blob/main/split_files/vae/qwen_image_vae.safetensors
https://huggingface.co/lightx2v/Qwen-Image-Lightning/resolve/main/Qwen-Image-Edit-2509/Qwen-Image-Edit-2509-Lightning-4steps-V1.0-bf16.safetensors

**Структура папок для моделей:**
```
ComfyUI/
└── models/
    ├── diffusion_models/
    │   └── qwen_image_edit_2509_fp8_e4m3fn.safetensors
    ├── loras/
    │   └── Qwen-Image-Lightning-4steps-V1.0.safetensors
    ├── vae/
    │   └── qwen_image_vae.safetensors
    └── text_encoders/
        └── qwen_2.5_vl_7b_fp8_scaled.safetensors
```

**Запуск ComfyUI:**
```bash
python main.py --listen 127.0.0.1 --port 8188
```
*Примечание: Если у вас нет мощного GPU, используйте флаг `--cpu` для запуска на процессоре.*

---

### 2. Развертывание сервиса генерации (Pixel-generation-service)

```bash
cd api
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
python api.py
```