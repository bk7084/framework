import numbers
from enum import Enum
import numpy as np


def sign(n: numbers.Number):
    if n > 0:
        return 1
    else:
        return -1


def hex2rgb(color):
    _str = color.lstrip('#')
    return [float(int(_str[i:i + 2], 16)) / 255. for i in (0, 2, 4)]


def hex2rgba(color, alpha):
    _str = color.lstrip('#')
    return [float(int(_str[i:i + 2], 16)) / 255. for i in (0, 2, 4)] + [alpha]


class Color:
    """
    Represents a color using 4 normalised channels (red, green, blue and alpha).

    Examples:
        >>> c0 = Color()
        >>> c1 = Color('#aabbcc', alpha = 1.0)
        >>> c2 = Color((1.0, 0.2, 0.3), alpha=1.0)
        >>> c3 = Color([1.0, 0.2, 0.3], alpha=1.0)
    """
    def __init__(self, color=None, alpha=1.0):
        """
        Stores the values(between 0.0 and 1.0) of a RGBA color.

        Args:
            color (str, list, tuple or np.ndarray):
                Input color could be a hex string, a tuple or np.ndarray.
            alpha (float):
                Alpha channel of the color.
        """
        if isinstance(color, (list, tuple, np.ndarray)):
            if len(color) < 3:
                raise ValueError("Color needs at least 3 elements.")
            self._rgba = np.ones(4, dtype=np.float32) * (0., 0., 0., alpha)
            self._rgba[...] = color[...]
        elif isinstance(color, str):
            self._rgba = np.asarray(hex2rgba(color, alpha))
        else:
            self._rgba = np.zeros(4, dtype=np.float32)

    @property
    def rgba(self):
        return self._rgba

    @property
    def rgb(self):
        return self._rgba[:3]

    def __getitem__(self, index):
        if index > 4 or index < 0:
            raise IndexError("Color has only 4 elements.")
        return self._rgba[index]

    def __setitem__(self, index, value):
        if index > 4 or index < 0:
            raise IndexError("Color has only 4 elements.")
        self._rgba[index] = np.clip(value, 0.0, 1.0)

    def __iter__(self):
        return self._rgba.__iter__()

    def __next__(self):
        self._index += 1
        if self._index >= len(self._rgba):
            self._index = -1
            raise StopIteration
        else:
            return self._rgba[self._index]


class PaletteMaterial(Enum):
    Red50 = '#ffebee'
    Red100 = '#ffcdd2'
    Red200 = '#ef9a9a'
    Red300 = '#e57373'
    Red400 = '#ef5350'
    Red500 = '#f44336'
    Red600 = '#e53935'
    Red700 = '#d32f2f'
    Red800 = '#c62828'
    Red900 = '#b71c1c'
    RedAccent100 = '#ff8a80'
    RedAccent200 = '#ff5252'
    RedAccent400 = '#ff1744'
    RedAccent700 = '#d50000'
    Pink50 = '#fce4ec'
    Pink100 = '#f8bbd0'
    Pink200 = '#f48fb1'
    Pink300 = '#f06292'
    Pink400 = '#ec407a'
    Pink500 = '#e91e63'
    Pink600 = '#d81b60'
    Pink700 = '#c2185b'
    Pink800 = '#ad1457'
    Pink900 = '#880e4f'
    PinkAccent100 = '#ff80ab'
    PinkAccent200 = '#ff4081'
    PinkAccent400 = '#f50057'
    PinkAccent700 = '#c51162'
    Purple50 = '#f3e5f5'
    Purple100 = '#e1bee7'
    Purple200 = '#ce93d8'
    Purple300 = '#ba68c8'
    Purple400 = '#ab47bc'
    Purple500 = '#9c27b0'
    Purple600 = '#8e24aa'
    Purple700 = '#7b1fa2'
    Purple800 = '#6a1b9a'
    Purple900 = '#4a148c'
    PurpleAccent100 = '#ea80fc'
    PurpleAccent200 = '#e040fb'
    PurpleAccent400 = '#d500f9'
    PurpleAccent700 = '#aa00ff'
    DeepPurple50 = '#ede7f6'
    DeepPurple100 = '#d1c4e9'
    DeepPurple200 = '#b39ddb'
    DeepPurple300 = '#9575cd'
    DeepPurple400 = '#7e57c2'
    DeepPurple500 = '#673ab7'
    DeepPurple600 = '#5e35b1'
    DeepPurple700 = '#512da8'
    DeepPurple800 = '#4527a0'
    DeepPurple900 = '#311b92'
    DeepPurpleAccent100 = '#b388ff'
    DeepPurpleAccent200 = '#7c4dff'
    DeepPurpleAccent400 = '#651fff'
    DeepPurpleAccent700 = '#6200ea'
    Indigo50 = '#e8eaf6'
    Indigo100 = '#c5cae9'
    Indigo200 = '#9fa8da'
    Indigo300 = '#7986cb'
    Indigo400 = '#5c6bc0'
    Indigo500 = '#3f51b5'
    Indigo600 = '#3949ab'
    Indigo700 = '#303f9f'
    Indigo800 = '#283593'
    Indigo900 = '#1a237e'
    IndigoAccent100 = '#8c9eff'
    IndigoAccent200 = '#536dfe'
    IndigoAccent400 = '#3d5afe'
    IndigoAccent700 = '#304ffe'
    Blue50 = '#e3f2fd'
    Blue100 = '#bbdefb'
    Blue200 = '#90caf9'
    Blue300 = '#64b5f6'
    Blue400 = '#42a5f5'
    Blue500 = '#2196f3'
    Blue600 = '#1e88e5'
    Blue700 = '#1976d2'
    Blue800 = '#1565c0'
    Blue900 = '#0d47a1'
    BlueAccent100 = '#82b1ff'
    BlueAccent200 = '#448aff'
    BlueAccent400 = '#2979ff'
    BlueAccent700 = '#2962ff'
    LightBlue50 = '#e1f5fe'
    LightBlue100 = '#b3e5fc'
    LightBlue200 = '#81d4fa'
    LightBlue300 = '#4fc3f7'
    LightBlue400 = '#29b6f6'
    LightBlue500 = '#03a9f4'
    LightBlue600 = '#039be5'
    LightBlue700 = '#0288d1'
    LightBlue800 = '#0277bd'
    LightBlue900 = '#01579b'
    LightBlueAccent100 = '#80d8ff'
    LightBlueAccent200 = '#40c4ff'
    LightBlueAccent400 = '#00b0ff'
    LightBlueAccent700 = '#0091ea'
    Cyan50 = '#e0f7fa'
    Cyan100 = '#b2ebf2'
    Cyan200 = '#80deea'
    Cyan300 = '#4dd0e1'
    Cyan400 = '#26c6da'
    Cyan500 = '#00bcd4'
    Cyan600 = '#00acc1'
    Cyan700 = '#0097a7'
    Cyan800 = '#00838f'
    Cyan900 = '#006064'
    CyanAccent100 = '#84ffff'
    CyanAccent200 = '#18ffff'
    CyanAccent400 = '#00e5ff'
    CyanAccent700 = '#00b8d4'
    Teal50 = '#e0f2f1'
    Teal100 = '#b2dfdb'
    Teal200 = '#80cbc4'
    Teal300 = '#4db6ac'
    Teal400 = '#26a69a'
    Teal500 = '#009688'
    Teal600 = '#00897b'
    Teal700 = '#00796b'
    Teal800 = '#00695c'
    Teal900 = '#004d40'
    TealAccent100 = '#a7ffeb'
    TealAccent200 = '#64ffda'
    TealAccent400 = '#1de9b6'
    TealAccent700 = '#00bfa5'
    Green50 = '#e8f5e9'
    Green100 = '#c8e6c9'
    Green200 = '#a5d6a7'
    Green300 = '#81c784'
    Green400 = '#66bb6a'
    Green500 = '#4caf50'
    Green600 = '#43a047'
    Green700 = '#388e3c'
    Green800 = '#2e7d32'
    Green900 = '#1b5e20'
    GreenAccent100 = '#b9f6ca'
    GreenAccent200 = '#69f0ae'
    GreenAccent400 = '#00e676'
    GreenAccent700 = '#00c853'
    LightGreen50 = '#f1f8e9'
    LightGreen100 = '#dcedc8'
    LightGreen200 = '#c5e1a5'
    LightGreen300 = '#aed581'
    LightGreen400 = '#9ccc65'
    LightGreen500 = '#8bc34a'
    LightGreen600 = '#7cb342'
    LightGreen700 = '#689f38'
    LightGreen800 = '#558b2f'
    LightGreen900 = '#33691e'
    LightGreenAccent100 = '#ccff90'
    LightGreenAccent200 = '#b2ff59'
    LightGreenAccent400 = '#76ff03'
    LightGreenAccent700 = '#64dd17'
    Lime50 = '#f9fbe7'
    Lime100 = '#f0f4c3'
    Lime200 = '#e6ee9c'
    Lime300 = '#dce775'
    Lime400 = '#d4e157'
    Lime500 = '#cddc39'
    Lime600 = '#c0ca33'
    Lime700 = '#afb42b'
    Lime800 = '#9e9d24'
    Lime900 = '#827717'
    LimeAccent100 = '#f4ff81'
    LimeAccent200 = '#eeff41'
    LimeAccent400 = '#c6ff00'
    LimeAccent700 = '#aeea00'
    Yellow50 = '#fffde7'
    Yellow100 = '#fff9c4'
    Yellow200 = '#fff59d'
    Yellow300 = '#fff176'
    Yellow400 = '#ffee58'
    Yellow500 = '#ffeb3b'
    Yellow600 = '#fdd835'
    Yellow700 = '#fbc02d'
    Yellow800 = '#f9a825'
    Yellow900 = '#f57f17'
    YellowAccent100 = '#ffff8d'
    YellowAccent200 = '#ffff00'
    YellowAccent400 = '#ffea00'
    YellowAccent700 = '#ffd600'
    Amber50 = '#fff8e1'
    Amber100 = '#ffecb3'
    Amber200 = '#ffe082'
    Amber300 = '#ffd54f'
    Amber400 = '#ffca28'
    Amber500 = '#ffc107'
    Amber600 = '#ffb300'
    Amber700 = '#ffa000'
    Amber800 = '#ff8f00'
    Amber900 = '#ff6f00'
    AmberAccent100 = '#ffe57f'
    AmberAccent200 = '#ffd740'
    AmberAccent400 = '#ffc400'
    AmberAccent700 = '#ffab00'
    Orange50 = '#fff3e0'
    Orange100 = '#ffe0b2'
    Orange200 = '#ffcc80'
    Orange300 = '#ffb74d'
    Orange400 = '#ffa726'
    Orange500 = '#ff9800'
    Orange600 = '#fb8c00'
    Orange700 = '#f57c00'
    Orange800 = '#ef6c00'
    Orange900 = '#e65100'
    OrangeAccent100 = '#ffd180'
    OrangeAccent200 = '#ffab40'
    OrangeAccent400 = '#ff9100'
    OrangeAccent700 = '#ff6d00'
    DeepOrange50 = '#fbe9e7'
    DeepOrange100 = '#ffccbc'
    DeepOrange200 = '#ffab91'
    DeepOrange300 = '#ff8a65'
    DeepOrange400 = '#ff7043'
    DeepOrange500 = '#ff5722'
    DeepOrange600 = '#f4511e'
    DeepOrange700 = '#e64a19'
    DeepOrange800 = '#d84315'
    DeepOrange900 = '#bf360c'
    DeepOrangeAccent100 = '#ff9e80'
    DeepOrangeAccent200 = '#ff6e40'
    DeepOrangeAccent400 = '#ff3d00'
    DeepOrangeAccent700 = '#dd2c00'
    Brown50 = '#efebe9'
    Brown100 = '#d7ccc8'
    Brown200 = '#bcaaa4'
    Brown300 = '#a1887f'
    Brown400 = '#8d6e63'
    Brown500 = '#795548'
    Brown600 = '#6d4c41'
    Brown700 = '#5d4037'
    Brown800 = '#4e342e'
    Brown900 = '#3e2723'
    Grey50 = '#fafafa'
    Grey100 = '#f5f5f5'
    Grey200 = '#eeeeee'
    Grey300 = '#e0e0e0'
    Grey400 = '#bdbdbd'
    Grey500 = '#9e9e9e'
    Grey600 = '#757575'
    Grey700 = '#616161'
    Grey800 = '#424242'
    Grey900 = '#212121'
    BlueGrey50 = '#eceff1'
    BlueGrey100 = '#cfd8dc'
    BlueGrey200 = '#b0bec5'
    BlueGrey300 = '#90a4ae'
    BlueGrey400 = '#78909c'
    BlueGrey500 = '#607d8b'
    BlueGrey600 = '#546e7a'
    BlueGrey700 = '#455a64'
    BlueGrey800 = '#37474f'
    BlueGrey900 = '#263238'
    Black = '#000000'
    White = '#ffffff'

    def as_color(self, alpha=1.0):
        return Color(self.value, alpha)

    def __iter__(self):
        return Color(self.value, 1.0).__iter__()


class PaletteWeb(Enum):
    Black = '#000000'
    Silver = '#c0c0c0'
    Gray = '#808080'
    White = '#ffffff'
    Maroon = '#800000'
    Red = '#ff0000'
    Purple = '#800080'
    Fuchsia = '#ff00ff'
    Green = '#008000'
    Lime = '#00ff00'
    Olive = '#808000'
    Yellow = '#ffff00'
    Navy = '#000080'
    Blue = '#0000ff'
    Teal = '#008080'
    Aqua = '#00ffff'

    def as_color(self, alpha=1.0):
        return Color(self.value, alpha)

    def __iter__(self):
        return Color(self.value, 1.0).__iter__()


class PaletteSvg(Enum):
    AliceBlue = '#f0f8ff'
    AntiqueWhite = '#faebd7'
    Aqua = '#00ffff'
    Aquamarine = '#7fffd4'
    Azure = '#f0ffff'
    Beige = '#f5f5dc'
    Bisque = '#ffe4c4'
    Black = '#000000'
    BlanchedAlmond = '#ffebcd'
    Blue = '#0000ff'
    BlueViolet = '#8a2be2'
    Brown = '#a52a2a'
    Burlywood = '#deb887'
    CadetBlue = '#5f9ea0'
    Chartreuse = '#7fff00'
    Chocolate = '#d2691e'
    Coral = '#ff7f50'
    CornflowerBlue = '#6495ed'
    CornSilk = '#fff8dc'
    Crimson = '#dc143c'
    Cyan = '#00ffff'
    DarkBlue = '#00008b'
    DarkCyan = '#008b8b'
    DarkGoldenRod = '#b8860b'
    DarkGray = '#a9a9a9'
    DarkGrey = '#a9a9a9'
    DarkGreen = '#006400'
    DarkKhaki = '#bdb76b'
    DarkMagenta = '#8b008b'
    DarkOliveGreen = '#556b2f'
    DarkOrange = '#ff8c00'
    DarkOrchid = '#9932cc'
    DarkRed = '#8b0000'
    DarkSalmon = '#e9967a'
    DarkSeaGreen = '#8fbc8f'
    DarkSlateBlue = '#483d8b'
    DarkSlateGray = '#2f4f4f'
    DarkSlateGrey = '#2f4f4f'
    DarkTurquoise = '#00ced1'
    DarkViolet = '#9400d3'
    DeepPink = '#ff1493'
    DeepSkyBlue = '#00bfff'
    DimGray = '#696969'
    DimGrey = '#696969'
    DodgerBlue = '#1e90ff'
    Firebrick = '#b22222'
    FloralWhite = '#fffaf0'
    ForestGreen = '#228b22'
    Fuchsia = '#ff00ff'
    Gainsboro = '#dcdcdc'
    GhostWhite = '#f8f8ff'
    Gold = '#ffd700'
    GoldenRod = '#daa520'
    Gray = '#808080'
    Grey = '#808080'
    Green = '#008000'
    GreenYellow = '#adff2f'
    Honeydew = '#f0fff0'
    HotPink = '#ff69b4'
    IndianRed = '#cd5c5c'
    Indigo = '#4b0082'
    Ivory = '#fffff0'
    Khaki = '#f0e68c'
    Lavender = '#e6e6fa'
    LavenderBlush = '#fff0f5'
    LawnGreen = '#7cfc00'
    LemonChiffon = '#fffacd'
    LightBlue = '#add8e6'
    LightCoral = '#f08080'
    LightCyan = '#e0ffff'
    LightGoldenRodYellow = '#fafad2'
    LightGray = '#d3d3d3'
    LightGrey = '#d3d3d3'
    LightGreen = '#90ee90'
    LightPink = '#ffb6c1'
    LightSalmon = '#ffa07a'
    LightSeaGreen = '#20b2aa'
    LightSkyBlue = '#87cefa'
    LightSlateGray = '#778899'
    LightSlateGrey = '#778899'
    LightSteelBlue = '#b0c4de'
    LightYellow = '#ffffe0'
    Lime = '#00ff00'
    LimeGreen = '#32cd32'
    Linen = '#faf0e6'
    Magenta = '#ff00ff'
    Maroon = '#800000'
    MediumAquamarine = '#66cdaa'
    MediumBlue = '#0000cd'
    MediumOrchid = '#ba55d3'
    MediumPurple = '#9370d8'
    MediumSeaGreen = '#3cb371'
    MediumSlateBlue = '#7b68ee'
    MediumSpringGreen = '#00fa9a'
    MediumTurquoise = '#48d1cc'
    MediumVioletRed = '#c71585'
    MidnightBlue = '#191970'
    MintCream = '#f5fffa'
    MistyRose = '#ffe4e1'
    Moccasin = '#ffe4b5'
    NavajoWhite = '#ffdead'
    Navy = '#000080'
    Oldlace = '#fdf5e6'
    Olive = '#808000'
    OlivedRab = '#6b8e23'
    Orange = '#ffa500'
    OrangeRed = '#ff4500'
    Orchid = '#da70d6'
    PaleGoldenRod = '#eee8aa'
    PaleGreen = '#98fb98'
    PaleTurquoise = '#afeeee'
    PaleVioletRed = '#d87093'
    Papayawhip = '#ffefd5'
    Peachpuff = '#ffdab9'
    Peru = '#cd853f'
    Pink = '#ffc0cb'
    Plum = '#dda0dd'
    PowderBlue = '#b0e0e6'
    Purple = '#800080'
    Red = '#ff0000'
    RosyBrown = '#bc8f8f'
    RoyalBlue = '#4169e1'
    SaddleBrown = '#8b4513'
    Salmon = '#fa8072'
    SandyBrown = '#f4a460'
    SeaGreen = '#2e8b57'
    Seashell = '#fff5ee'
    Sienna = '#a0522d'
    Silver = '#c0c0c0'
    SkyBlue = '#87ceeb'
    SlateBlue = '#6a5acd'
    SlateGray = '#708090'
    SlateGrey = '#708090'
    Snow = '#fffafa'
    SpringGreen = '#00ff7f'
    SteelBlue = '#4682b4'
    Tan = '#d2b48c'
    Teal = '#008080'
    Thistle = '#d8bfd8'
    Tomato = '#ff6347'
    Turquoise = '#40e0d0'
    Violet = '#ee82ee'
    Wheat = '#f5deb3'
    White = '#ffffff'
    WhiteSmoke = '#f5f5f5'
    Yellow = '#ffff00'
    YellowGreen = '#9acd32'

    def as_color(self, alpha=1.0):
        return Color(self.value, alpha)

    def __iter__(self):
        return Color(self.value, 1.0).__iter__()


class PaletteDefault(Enum):
    Background = '#1c1c1c'
    BlackA = '#3d352a'
    BlackB = '#554444'
    RedA = '#cd5c5c'
    RedB = '#cc5533'
    GreenA = '#86af80'
    GreenB = '#88aa22'
    YellowA = '#e8ae5b'
    YellowB = '#ffa75d'
    BlueA = '#6495ed'
    BlueB = '#87ceeb'
    BrownA = '#996600'
    BrownB = '#deb887'
    GreyA = '#b0c4de'
    GreyB = '#d0d0d0'
    WhiteA = '#bbaa99'
    WhiteB = '#ddccbb'

    def as_color(self, alpha=1.0):
        return Color(self.value, alpha)

    def __iter__(self):
        return Color(self.value, 1.0).__iter__()
