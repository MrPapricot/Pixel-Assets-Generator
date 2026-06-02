from typing import Dict, Any
from utils import PixelArtTransformer


class TransformerService:
    """Сервис для работы с PixelArtTransformer"""

    @staticmethod
    def process_image(config: Dict[str, Any], image_path: str, result_path: str) -> bool:
        """Обработка изображения"""
        transformer = PixelArtTransformer(config)
        return transformer.process_image(image_path, result_path)