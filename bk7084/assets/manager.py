import enum
import logging

from .resolver import default_resolver
from .image import Image


class AssetKind(enum.Enum):
    Image = 0
    Texture = 1
    Material = 2


class AssetManager:
    def __init__(self, resolver=default_resolver):
        self._resolver = resolver
        self._materials = []
        self._textures = []
        self._images = {}

    def get_or_load_image(self, image_path):
        """
        Returns the requested image. Load the image if it doesn't exist.

        Args:
            image_path (str):

        Returns:
            Pillow.Image
        """
        if image_path not in self._images:
            logging.info(f'Create image {image_path}')
            image = Image.open(image_path)
            self._images[image_path] = image

        return self._images[image_path]


default_asset_mgr: AssetManager = AssetManager()
