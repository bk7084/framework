from PIL import Image as PILImage


class Image:
    def __init__(self, pil_image):
        self._pil_image = pil_image
        self._num_channels = len(pil_image.getbands())

    @staticmethod
    def open(file_path):
        img = PILImage.open(file_path).convert('RGBA').transpose(PILImage.FLIP_TOP_BOTTOM)
        return Image(img)

    @property
    def num_channels(self):
        return self._num_channels

    @property
    def raw_data(self):
        return self._pil_image.tobytes()

    @property
    def pil_image(self):
        return self._pil_image

    @property
    def width(self):
        return self._pil_image.width

    @property
    def height(self):
        return self._pil_image.height
