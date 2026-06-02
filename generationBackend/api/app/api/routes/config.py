from fastapi import APIRouter

router = APIRouter(prefix="/api/v1/config", tags=["Configuration"])

@router.get("/presets")
async def get_config_presets():
    """Получение предустановленных конфигураций"""
    presets = {
        "pixel_character": {
            "technical": {
                "base_resolution": "32x32",
                "color_mode": "limited_16",
                "outline": "black_outline",
                "width": 8,
                "height": 8
            },
            "visual": {
                "stylization": "clean",
                "lighting": "cell_shaded",
                "projection": "side_view"
            },
            "functional": {
                "asset_type": "character"
            }
        },
        "isometric_prop": {
            "technical": {
                "base_resolution": "64x64",
                "color_mode": "limited_32",
                "outline": "colored_outline",
                "width": 16,
                "height": 16
            },
            "visual": {
                "stylization": "high_detail",
                "lighting": "smooth",
                "projection": "isometric_2_1"
            },
            "functional": {
                "asset_type": "prop"
            }
        },
        "minimalist_tile": {
            "technical": {
                "base_resolution": "16x16",
                "color_mode": "limited_8",
                "outline": "no_outline",
                "width": 4,
                "height": 4
            },
            "visual": {
                "stylization": "minimalist",
                "lighting": "flat",
                "projection": "top_down"
            },
            "functional": {
                "asset_type": "tile"
            }
        }
    }
    return presets