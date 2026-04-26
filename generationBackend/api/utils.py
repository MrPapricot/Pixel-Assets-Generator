import base64
import io
import json
import os
import sys
import time
import requests
from rembg import remove, new_session
from PIL import Image, ImageDraw
import numpy as np

# Возможные значения параметров:
# Технические:
# base_resolution: "16x16", "24x24", "32x32", "48x48", "64x64", "128x128"
#
# color_mode: "monochrome", "limited_4", "limited_8", "limited_16", "limited_32", "limited_256", "true_color"
#
# outline: "black_outline", "colored_outline", "no_outline"
#
# Визуальные:
# stylization: "minimalist", "clean", "dithered", "high_detail"
#
# lighting: "flat", "cell_shaded", "smooth"
#
# projection: "side_view", "top_down", "isometric_2_1", "isometric_1_1"
#
# Функциональные:
# asset_type: "sprite", "object", "tile"

class PixelArtTransformer:
    def __init__(self, config_input):
        """Инициализация с загрузкой конфигурации из JSON"""
        self.config = config_input
        self.validate_config()

    def validate_config(self):
        """Валидация параметров конфигурации"""
        required_sections = ['technical', 'visual', 'functional']
        print(self.config)
        for section in required_sections:
            if section not in self.config:
                raise ValueError(f"Отсутствует секция '{section}' в конфигурации")
        
        # Проверка технических параметров
        tech = self.config['technical']
        valid_resolutions = ['16x16', '24x24', '32x32', '48x48', '64x64', '128x128']
        if tech.get('base_resolution') not in valid_resolutions:
            raise ValueError(f"Некорректное разрешение. Допустимые: {valid_resolutions}")
        
        valid_color_modes = ['monochrome', 'limited_4', 'limited_8', 'limited_16', 'limited_32', 'limited_256', 'true_color']
        if tech.get('color_mode') not in valid_color_modes:
            raise ValueError(f"Некорректный цветовой режим. Допустимые: {valid_color_modes}")
        
        valid_outlines = ['black_outline', 'colored_outline', 'no_outline']
        if tech.get('outline') not in valid_outlines:
            raise ValueError(f"Некорректный тип контура. Допустимые: {valid_outlines}")
        
        # Проверка визуальных параметров
        vis = self.config['visual']
        valid_stylizations = ['minimalist', 'clean', 'dithered', 'high_detail']
        if vis.get('stylization') not in valid_stylizations:
            raise ValueError(f"Некорректная стилизация. Допустимые: {valid_stylizations}")
        
        valid_lighting = ['flat', 'cell_shaded', 'smooth']
        if vis.get('lighting') not in valid_lighting:
            raise ValueError(f"Некорректное освещение. Допустимые: {valid_lighting}")
        
        valid_projections = ['side_view', 'top_down', 'isometric_2_1', 'isometric_1_1']
        if vis.get('projection') not in valid_projections:
            raise ValueError(f"Некорректная проекция. Допустимые: {valid_projections}")
        
        # Проверка функциональных параметров
        func = self.config['functional']
        valid_asset_types = ["sprite", "object", "tile"]
        if func.get('asset_type') not in valid_asset_types:
            raise ValueError(f"Некорректный тип ассета. Допустимые: {valid_asset_types}")
        
        print("✅ Конфигурация успешно проверена")
    
    def process_image(self, input_path, output_path):
        """Основной метод обработки изображения"""
        print(f"\n🖼️  Обработка изображения: {input_path}")
        
        # Загрузка изображения
        try:
            img = Image.open(input_path).convert("RGBA")
        except Exception as e:
            print(f"❌ Ошибка загрузки изображения: {e}")
            return False
        
        # Получаем параметры
        tech = self.config['technical']
        vis = self.config['visual']
        func = self.config['functional']
        
        # Удаление фона
        img = self.remove_background(img)
        
        # Применение проекции
        projection = vis['projection']
        img = self.apply_projection(img, projection, func['asset_type']) #!!!!!!!!!!!!!!!!!!!!!!!

        # Применение освещения
        lighting = vis['lighting']
        img = self.apply_lighting(img, lighting)

        # Применение стилизации
        stylization = vis['stylization']
        img = self.apply_stylization(img, stylization) #!!!!!!!!!!!!!!!!!!!!!!!

        # Пикселизация
        resolution = tech['base_resolution']
        tile_size = list(map(int, resolution.split('x')))[0]
        width = tech['width']
        height = tech['height']
        img = self.resize_to_resolution(img, tile_size, width, height)

        # Применение цветового режима
        color_mode = tech['color_mode']
        img = self.apply_color_mode(img, color_mode)

        # Применение контура
        outline = tech['outline']
        img = self.apply_outline(img, outline)

        # Сохранение результата
        img.save(output_path, format='PNG')
        print(f"✅ Результат сохранен: {output_path}")
        print(f"📊 Итоговый размер: {img.size[0]}x{img.size[1]}")
        
        return True

    def resize_to_resolution(self, img, tile_size, width, height):
        """
        Изменение размера изображения до заданного количества тайлов

        Args:
            img: PIL Image
            tile_size: размер одного тайла в пикселях (например, 16, 32, 64)
            width: количество тайлов по ширине
            height: количество тайлов по высоте

        Returns:
            PIL Image с размером (width * tile_size) x (height * tile_size)
        """
        if img.mode != 'RGBA':
            img = img.convert('RGBA')

        # Получаем альфа-канал
        alpha = img.getchannel('A')

        # Конвертируем в numpy массив для быстрой обработки
        alpha_array = np.array(alpha)

        # Находим непустые пиксели (альфа > threshold)
        non_empty_pixels = np.where(alpha_array > 0)

        # Если все пиксели пустые - возвращаем исходное изображение
        if len(non_empty_pixels[0]) == 0:
            print("⚠️ Изображение полностью пустое!")
            return img

        # Находим границы
        top = non_empty_pixels[0].min()
        bottom = non_empty_pixels[0].max() + 1
        left = non_empty_pixels[1].min()
        right = non_empty_pixels[1].max() + 1

        # Обрезаем
        bbox = (left, top, right, bottom)
        img = img.crop(bbox)

        # Вычисляем целевой размер в пикселях
        target_width = width * tile_size
        target_height = height * tile_size

        # Сохраняем пропорции исходного изображения
        img_ratio = img.width / img.height
        target_ratio = target_width / target_height

        # Определяем, как масштабировать (fit или fill)
        if img_ratio > target_ratio:
            # Изображение шире - подгоняем по ширине
            new_width = target_width
            new_height = int(target_width / img_ratio)
        else:
            # Изображение выше - подгоняем по высоте
            new_height = target_height
            new_width = int(target_height * img_ratio)

        # Масштабируем с сохранением пропорций
        img_resized = img.resize((new_width, new_height), Image.Resampling.NEAREST)

        # Создаем холст нужного размера и центрируем изображение
        result = Image.new('RGBA', (target_width, target_height), (0, 0, 0, 0))
        offset_x = (target_width - new_width) // 2
        offset_y = (target_height - new_height) // 2
        result.paste(img_resized, (offset_x, offset_y))

        return result

    def remove_background(self, input_image, model_name='u2net'):
        """
        Удаление фона
        Args:
            model_name: модель для удаления фона ('u2net', 'u2netp', 'u2net_human_seg')
        """
        # Создаем сессию с выбранной моделью
        session = new_session(model_name)

        # Удаляем фон
        output_image = remove(
            input_image,
            session=session,
            post_process_mask=True,  # Улучшает качество маски
            alpha_matting=False,  # Использовать alpha matting для краев
            alpha_matting_foreground_threshold=240,
            alpha_matting_background_threshold=10,
            alpha_matting_erode_structure_size=10
        )

        return output_image

    def apply_stylization(self, img, stylization):
        """
        Применение стилизации к изображению через ComfyUI + Qwen Image Edit

        Args:
            img: PIL Image
            stylization: тип стилизации ('minimalist', 'clean', 'dithered', 'high_detail')
            asset_type: тип ассета ('character', 'prop', 'tile')

        Returns:
            Стилизованное PIL Image
        """
        if img is None:
            return None

        # Формируем промпт в зависимости от типа стилизации
        stylization_prompts = {
            "minimalist": (
                "Transform the image into minimalist pixel art style. "
                "Reduce details to essential shapes only. "
                "Use very limited color palette (4-8 colors maximum). "
                "Clean silhouettes, no unnecessary details, iconic representation. "
                "Flat colors, sharp pixel edges, retro game aesthetic."
            ),
            "clean": (
                "Transform the image into clean pixel art style. "
                "Smooth color gradients replaced with flat color clusters. "
                "Crisp pixel edges, well-defined outlines, organized color palette. "
                "Modern indie game aesthetic, polished look, no noise or dithering. "
                "Clean shading with 2-3 tones per color."
            ),
            "dithered": (
                "Transform the image into dithered pixel art style. "
                "Apply classic dithering patterns (Bayer matrix, 50% threshold). "
                "Use checkerboard patterns for gradients and shadows. "
                "Retro 16-bit era aesthetic, DOS game style. "
                "Visible dithering artifacts, limited color palette with pattern blending. "
                "Nostalgic 90s computer graphics feel."
            ),
            "high_detail": (
                "Transform the image into high-detail pixel art style. "
                "Preserve and enhance fine details and textures. "
                "Rich color palette with subtle variations. "
                "Intricate pixel-level detailing, sharp highlights. "
                "Modern high-resolution pixel art, almost illustrative quality. "
                "Complex shading with multiple color tones, anti-aliasing on edges."
            )
        }

        full_prompt = stylization_prompts.get(stylization, stylization_prompts["clean"])

        print(f"\n🎨 Применение стилизации: {stylization}")
        print(f"📝 Промпт: {full_prompt[:120]}...")

        # Вызываем ComfyUI
        try:
            result = self.call_comfyui_api(img, full_prompt, f"stylization_{stylization}")

            if result and result != img:
                print(f"✅ Стилизация '{stylization}' применена успешно")
                return result
            else:
                print(f"⚠️ Не удалось применить стилизацию, возвращаю оригинал")
                return img

        except Exception as e:
            print(f"❌ Ошибка при вызове ComfyUI: {e}")
            return img
    
    def apply_outline(self, img, outline_type):
        """Применение контура к изображению"""
        if outline_type == "no_outline":
            return img
        
        width, height = img.size
        result = img.copy()
        draw = ImageDraw.Draw(result)
        
        # Получаем альфа-канал для определения границ
        alpha = img.getchannel('A')
        alpha_array = np.array(alpha)
        
        # Находим границы объекта
        kernel = np.array([[0, 1, 0], [1, 0, 1], [0, 1, 0]])
        
        # Создаем маску границ (где альфа > 0, но сосед с альфа == 0)
        boundary = np.zeros_like(alpha_array)
        for i in range(1, height-1):
            for j in range(1, width-1):
                if alpha_array[i, j] > 0:
                    # Проверяем соседей
                    if (alpha_array[i-1, j] == 0 or 
                        alpha_array[i+1, j] == 0 or 
                        alpha_array[i, j-1] == 0 or 
                        alpha_array[i, j+1] == 0):
                        boundary[i, j] = 255
        
        # Определяем цвет контура
        if outline_type == "black_outline":
            outline_color = (0, 0, 0, 255)
        else:  # colored_outline
            # Берем средний цвет объекта для границы
            img_array = np.array(img)
            mask = alpha_array > 0
            if np.any(mask):
                avg_color = np.mean(img_array[mask], axis=0)[:3]
                outline_color = tuple(int(c * 0.5) for c in avg_color) + (255,)
            else:
                outline_color = (0, 0, 0, 255)
        
        # Рисуем контур
        for i in range(height):
            for j in range(width):
                if boundary[i, j] == 255:
                    draw.point((j, i), fill=outline_color)
        
        return result

    def apply_color_mode(self, img, color_mode):
        """Применение цветового режима (только Pillow, без sklearn)"""
        if color_mode == "true_color":
            return img

        if color_mode == "monochrome":
            # Конвертируем в черно-белое через Pillow
            # 'L' - режим градаций серого
            gray_img = img.convert('L')
            # Бинаризация с порогом 128
            bw_img = gray_img.point(lambda x: 255 if x > 128 else 0, mode='1')
            # Возвращаем в RGB с альфа-каналом
            rgb_img = bw_img.convert('RGB')

            # Сохраняем альфа-канал если был
            if img.mode == 'RGBA':
                rgb_img.putalpha(img.getchannel('A'))
            elif img.mode == 'LA':
                rgb_img.putalpha(img.getchannel('A'))

            return rgb_img

        elif color_mode.startswith("limited_"):
            # Ограниченная палитра
            num_colors = int(color_mode.split('_')[1])

            # Используем встроенный метод quantize из Pillow
            # Для палитрового изображения (P mode)
            if img.mode == 'RGBA':
                # Pillow quantize не работает с RGBA напрямую, конвертируем в RGB
                rgb_img = img.convert('RGB')
                alpha = img.getchannel('A')
            elif img.mode == 'LA':
                rgb_img = img.convert('L').convert('RGB')
                alpha = img.getchannel('A')
            else:
                rgb_img = img.convert('RGB')
                alpha = None

            # Квантование до N цветов
            # method=0 - MEDIANCUT (быстрый и качественный)
            # method=1 - MAXCOVERAGE (медленнее, но лучше для маленьких палитр)
            # method=2 - FASTOCTREE (самый быстрый)
            quantized = rgb_img.quantize(
                colors=num_colors,
                method=2,  # FASTOCTREE - хорошо подходит для пиксель-арта
                kmeans=0,  # не используем k-means из Pillow (он экспериментальный)
                dither=Image.Dither.NONE  # БЕЗ дизеринга - важно для пиксель-арта!
            )

            # Конвертируем обратно в RGB
            result = quantized.convert('RGB')

            # Восстанавливаем альфа-канал если был
            if alpha is not None:
                result.putalpha(alpha)

            return result

        return img

    def call_comfyui_api(self, img: Image.Image, prompt: str, projection_type: str) -> Image.Image:
        """
        Отправляет изображение и промпт в ComfyUI API с workflow Qwen Image Edit
        """
        comfyui_url = self.config.get('comfyui', {}).get('url', "http://127.0.0.1:8188")

        # Путь к вашему workflow файлу
        script_dir = os.path.dirname(os.path.abspath(__file__))
        workflow_path = os.path.join(script_dir, "workflows", "qwen_projection_api.json")

        # 1. Загружаем workflow
        try:
            with open(workflow_path, 'r', encoding='utf-8') as f:
                workflow = json.load(f)
        except FileNotFoundError:
            print(f"❌ Файл workflow '{workflow_path}' не найден!")
            return img

        # 2. Загружаем изображение через upload API
        buffered = io.BytesIO()
        img.save(buffered, format="PNG")
        buffered.seek(0)

        # Уникальное имя файла чтобы избежать конфликтов
        filename = f"api_input_{projection_type}.png"

        upload_response = requests.post(
            f"{comfyui_url}/upload/image",
            files={"image": (filename, buffered, "image/png")},
            data={"overwrite": "true"}  # Перезаписывать если существует
        )

        if upload_response.status_code != 200:
            print(f"❌ Ошибка загрузки изображения: {upload_response.text}")
            return img

        uploaded_name = upload_response.json().get("name", filename)
        print(f"✅ Изображение загружено как '{uploaded_name}'")

        # 3. Модифицируем workflow
        if "78" in workflow:
            workflow["78"]["inputs"]["image"] = uploaded_name  # Просто имя файла
            print(f"✅ Имя файла установлено в ноду 78: {uploaded_name}")

        # Нода 435: Prompt
        if "435" in workflow:
            workflow["435"]["inputs"]["value"] = prompt
            print(f"✅ Промпт установлен: '{prompt[:50]}...'")

        # Нода 433:443: Lightning LoRA
        use_lightning = self.config.get('comfyui', {}).get('use_lightning_lora', True)
        if "433:443" in workflow:
            workflow["433:443"]["inputs"]["value"] = use_lightning
            print(f"⚡ Lightning LoRA: {'включен' if use_lightning else 'выключен'}")

        # Случайный seed
        if "433:3" in workflow and self.config.get('comfyui', {}).get('random_seed', True):
            import random
            workflow["433:3"]["inputs"]["seed"] = random.randint(0, 2 ** 64 - 1)

        # 4. Отправляем запрос
        print(f"🚀 Отправка запроса в ComfyUI ({projection_type})...")

        try:
            response = requests.post(
                f"{comfyui_url}/prompt",
                json={"prompt": workflow},
                timeout=30
            )

            if response.status_code != 200:
                print(f"❌ Ошибка API ({response.status_code}): {response.text[:200]}")
                return img

            prompt_id = response.json()['prompt_id']
            print(f"✅ Запрос принят. ID: {prompt_id}")

            # 5. Ожидание результата
            result_img = self._wait_for_comfyui_result(comfyui_url, prompt_id)

            if result_img is None:
                print(f"❌ Не удалось получить результат от ComfyUI")
                return img

            return result_img

        except requests.exceptions.RequestException as e:
            print(f"❌ Ошибка сети: {e}")
            return img
        except Exception as e:
            print(f"❌ Неожиданная ошибка: {e}")
            return img

    def _wait_for_comfyui_result(self, comfyui_url: str, prompt_id: str, timeout: int = 10800) -> Image.Image:
        """
        Ожидание и получение результата из ComfyUI

        Args:
            comfyui_url: URL ComfyUI
            prompt_id: ID запроса
            timeout: таймаут в секундах

        Returns:
            Результирующее изображение или None
        """
        start_time = time.time()
        check_interval = 1  # начальный интервал проверки

        print(f"⏳ Ожидание генерации...")

        while time.time() - start_time < timeout:
            time.sleep(check_interval)

            # Прогрессивное увеличение интервала
            check_interval = min(check_interval + 0.5, 3)

            try:
                history_response = requests.get(
                    f"{comfyui_url}/history/{prompt_id}",
                    timeout=10
                )

                if history_response.status_code != 200:
                    continue

                history = history_response.json()

                # Проверяем, завершена ли генерация
                if prompt_id not in history:
                    continue

                prompt_history = history[prompt_id]

                # Проверяем статус
                if 'outputs' not in prompt_history:
                    # Возможно, генерация еще идет
                    if 'status' in prompt_history:
                        status = prompt_history['status']
                        if status['completed'] is False:
                            continue
                    continue

                # Генерация завершена, ищем выходное изображение
                outputs = prompt_history['outputs']

                # Нода 60: SaveImage (результат)
                if "60" in outputs:
                    images = outputs["60"].get("images", [])
                    if images:
                        image_info = images[0]
                        image_url = f"{comfyui_url}/view?filename={image_info['filename']}&subfolder={image_info['subfolder']}&type={image_info['type']}"

                        # Скачиваем результат
                        img_response = requests.get(image_url, timeout=30)
                        if img_response.status_code == 200:
                            elapsed = time.time() - start_time
                            print(f"✅ Генерация завершена за {elapsed:.1f} сек")
                            return Image.open(io.BytesIO(img_response.content)).convert("RGBA")

                # Если результат не найден в ноде 60, ищем в других нодах
                for node_id, node_output in outputs.items():
                    if 'images' in node_output:
                        image_info = node_output['images'][0]
                        image_url = f"{comfyui_url}/view?filename={image_info['filename']}&subfolder={image_info['subfolder']}&type={image_info['type']}"

                        img_response = requests.get(image_url, timeout=30)
                        if img_response.status_code == 200:
                            elapsed = time.time() - start_time
                            print(f"✅ Генерация завершена за {elapsed:.1f} сек (нода {node_id})")
                            return Image.open(io.BytesIO(img_response.content)).convert("RGBA")

                print(f"⚠️ Результат не найден в outputs")
                return None

            except requests.exceptions.RequestException as e:
                print(f"⚠️ Ошибка сети при проверке: {e}")
                continue
            except Exception as e:
                print(f"⚠️ Неожиданная ошибка: {e}")
                continue

        print(f"❌ Таймаут ожидания ({timeout} сек)")
        return None

    def apply_projection(self, img, projection, asset_type):
        """
        Применение проекции к изображению через ComfyUI + Qwen Image Edit
        """
        if img is None:
            return None

        # Формируем промпт в зависимости от проекции
        prompts = {
            "side_view": (
                "Transform the object to a strict side view (profile view). "
                "The object should be perfectly perpendicular to the viewer. "
            ),
            "top_down": (
                "Transform the object to a top-down view (bird's eye view). "
                "Looking directly from above. "
            ),
            "isometric_2_1": (
                "Transform the object to an isometric dimetric projection with 2:1 ratio. "
                "View from 30 degree angle. "
            ),
            "isometric_1_1": (
                "Transform the object to a true isometric projection (1:1 ratio). "
                "Perfect 30/60 degree angles, classic isometric. "
            )
        }

        base_prompt = prompts.get(projection, prompts["side_view"])

        # Добавляем контекст типа ассета
        asset_context = {
            "sprite": "Show the object on white background.",
            "object": "Show the object on white background.",
            "tile": "Tileable texture, seamless"
        }

        full_prompt = f"{base_prompt} Asset type: {asset_type}. {asset_context.get(asset_type, '')}"

        print(f"\n🎯 Применение проекции: {projection}")
        print(f"📝 Промпт: {full_prompt[:100]}...")

        # Вызываем ComfyUI
        try:
            result = self.call_comfyui_api(img, full_prompt, projection)

            if result and result != img:
                print(f"✅ Проекция '{projection}' применена успешно")
                return result
            else:
                print(f"⚠️ Не удалось применить проекцию, возвращаю оригинал")
                return img

        except Exception as e:
            print(f"❌ Ошибка при вызове ComfyUI: {e}")
            return img
    
    def apply_lighting(self, img, lighting):
        """Применение освещения"""
        if lighting == "flat":
            return img
            
        elif lighting == "cell_shaded":
            # Резкие тени
            img_array = np.array(img)
            alpha = img_array[..., 3]
            
            # Создаем градиент слева направо
            gradient = np.linspace(0.7, 1.0, img_array.shape[1])
            gradient = np.tile(gradient, (img_array.shape[0], 1))
            
            # Применяем градиент к RGB каналам
            for i in range(3):
                img_array[..., i] = (img_array[..., i] * gradient).astype(np.uint8)
            
            return Image.fromarray(img_array)
            
        elif lighting == "smooth":
            # Плавное освещение
            img_array = np.array(img)
            alpha = img_array[..., 3]
            
            # Создаем плавный радиальный градиент
            h, w = img_array.shape[:2]
            Y, X = np.ogrid[:h, :w]
            center_y, center_x = h // 2, w // 2
            
            dist = np.sqrt((X - center_x)**2 + (Y - center_y)**2)
            max_dist = np.sqrt(center_x**2 + center_y**2)
            smooth_grad = 0.7 + 0.3 * (1 - dist / max_dist)
            
            for i in range(3):
                img_array[..., i] = (img_array[..., i] * smooth_grad).astype(np.uint8)
            
            return Image.fromarray(img_array)
            
        return img


def load_config(config_file):
    """Загрузка JSON конфигурации"""
    try:
        # Ищем JSON в директории запуска
        script_dir = os.path.dirname(os.path.abspath(__file__))
        config_path = os.path.join(script_dir, "test", config_file)

        with open(config_path, 'r', encoding='utf-8') as f:
            config = json.load(f)
        print(f"✅ Конфигурация загружена из {config_path}")
        return config
    except FileNotFoundError:
        print(f"❌ Файл {config_file} не найден в {script_dir}")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"❌ Ошибка парсинга JSON: {e}")
        sys.exit(1)

def main():
    """Главная функция"""
    print("🎨 ПИКСЕЛЬ-АРТ ТРАНСФОРМАТОР")
    
    # Инициализация трансформатора
    transformer = PixelArtTransformer(load_config("config.json"))


    script_dir = os.path.dirname(os.path.abspath(__file__))
    input_path = os.path.join(script_dir, "test", "input.png")
    
    if input_path is None:
        print("❌ Не найдено входное изображение!")
        return
    
    output_path = os.path.join(script_dir,"test", "output.png")
    
    # Обработка изображения
    transformer.process_image(input_path, output_path)


if __name__ == "__main__":
    main()