from enum import Enum

class ColorMode(str, Enum):
    MONOCHROME = "monochrome"
    LIMITED_4 = "limited_4"
    LIMITED_8 = "limited_8"
    LIMITED_16 = "limited_16"
    LIMITED_32 = "limited_32"
    LIMITED_256 = "limited_256"
    TRUE_COLOR = "true_color"

class OutlineType(str, Enum):
    BLACK_OUTLINE = "black_outline"
    COLORED_OUTLINE = "colored_outline"
    NO_OUTLINE = "no_outline"

class StylizationType(str, Enum):
    MINIMALIST = "minimalist"
    CLEAN = "clean"
    DITHERED = "dithered"
    HIGH_DETAIL = "high_detail"

class LightingType(str, Enum):
    FLAT = "flat"
    CELL_SHADED = "cell_shaded"
    SMOOTH = "smooth"

class ProjectionType(str, Enum):
    SIDE_VIEW = "side_view"
    TOP_DOWN = "top_down"
    ISOMETRIC_2_1 = "isometric_2_1"
    ISOMETRIC_1_1 = "isometric_1_1"

class AssetType(str, Enum):
    CHARACTER = "character"
    PROP = "prop"
    TILE = "tile"

class BaseResolution(str, Enum):
    R16x16 = "16x16"
    R24x24 = "24x24"
    R32x32 = "32x32"
    R48x48 = "48x48"
    R64x64 = "64x64"
    R128x128 = "128x128"

class TaskStatusEnum(str, Enum):
    PENDING = "pending"
    PROCESSING = "processing"
    COMPLETED = "completed"
    FAILED = "failed"