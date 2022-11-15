from enum import IntEnum, IntFlag

import glfw


class MouseButton(IntFlag):
    NONE = 0
    Left = 1 << 1
    Middle = 1 << 2
    Right = 1 << 3

    @classmethod
    def from_glfw_mouse_btn_code(cls, btn_code):
        return _MOUSE_MAP[btn_code]


class KeyModifier(IntFlag):
    NONE = 0
    Shift = 1 << 0
    Ctrl = 1 << 1
    Alt = 1 << 2
    Super = 1 << 5

    @classmethod
    def from_glfw_modifiers(cls, mods):
        modifiers = cls.NONE
        if mods & glfw.MOD_SHIFT:
            modifiers |= cls.Shift
        if mods & glfw.MOD_CONTROL:
            modifiers |= cls.Ctrl
        if mods & glfw.MOD_ALT:
            modifiers |= cls.Alt
        if mods & glfw.MOD_SUPER:
            modifiers |= cls.Super
        return modifiers


class KeyCode(IntEnum):
    Space = 32
    Apostrophe = 39  # '
    Comma = 44  # ,
    Minus = 45  # -
    Period = 46  # .
    Slash = 47  # /
    Num0 = 48
    Num1 = 49
    Num2 = 50
    Num3 = 51
    Num4 = 52
    Num5 = 53
    Num6 = 54
    Num7 = 55
    Num8 = 56
    Num9 = 57
    Semicolon = 59  # ;
    Equal = 61  # =
    A = 65
    B = 66
    C = 67
    D = 68
    E = 69
    F = 70
    G = 71
    H = 72
    I = 73
    J = 74
    K = 75
    L = 76
    M = 77
    N = 78
    O = 79
    P = 80
    Q = 81
    R = 82
    S = 83
    T = 84
    U = 85
    V = 86
    W = 87
    X = 88
    Y = 89
    Z = 90
    LeftBracket = 91  # [
    Backslash = 92  # \
    RightBracket = 93  # ]
    GraveAccent = 96  # `
    Escape = 256
    Enter = 257
    Tab = 258
    Backspace = 259
    Delete = 261
    Right = 262
    Left = 263
    Down = 264
    Up = 265
    PageUp = 266
    PageDown = 267
    Home = 268
    End = 269
    CapsLock = 280
    F1 = 290
    F2 = 291
    F3 = 292
    F4 = 293
    F5 = 294
    F6 = 295
    F7 = 296
    F8 = 297
    F9 = 298
    F10 = 299
    F11 = 300
    F12 = 301
    Keypad0 = 320
    Keypad1 = 321
    Keypad2 = 322
    Keypad3 = 323
    Keypad4 = 324
    Keypad5 = 325
    Keypad6 = 326
    Keypad7 = 327
    Keypad8 = 328
    Keypad9 = 329
    KeypadDecimal = 330
    KeypadDivide = 331
    KeypadMultiply = 332
    KeypadSubtract = 333
    KeypadAdd = 334
    KeypadEnter = 335
    KeypadEqual = 336
    LeftShift = 1 << 9
    LeftControl = 1 << 10
    LeftAlt = 1 << 11
    LeftSuper = 1 << 12
    RightShift = 1 << 13
    RightControl = 1 << 14
    RightAlt = 1 << 15
    RightSuper = 1 << 16
    Menu = 348

    @classmethod
    def from_glfw_keycode(cls, keycode):
        if keycode in _KEY_MAP:
            return _KEY_MAP[keycode]


_KEY_MAP = {
    glfw.KEY_SPACE: KeyCode.Space,
    glfw.KEY_APOSTROPHE: KeyCode.Apostrophe,
    glfw.KEY_COMMA: KeyCode.Comma,
    glfw.KEY_MINUS: KeyCode.Minus,
    glfw.KEY_PERIOD: KeyCode.Period,
    glfw.KEY_SLASH: KeyCode.Slash,
    glfw.KEY_0: KeyCode.Num0,
    glfw.KEY_1: KeyCode.Num1,
    glfw.KEY_2: KeyCode.Num2,
    glfw.KEY_3: KeyCode.Num3,
    glfw.KEY_4: KeyCode.Num4,
    glfw.KEY_5: KeyCode.Num5,
    glfw.KEY_6: KeyCode.Num6,
    glfw.KEY_7: KeyCode.Num7,
    glfw.KEY_8: KeyCode.Num8,
    glfw.KEY_9: KeyCode.Num9,
    glfw.KEY_SEMICOLON: KeyCode.Semicolon,
    glfw.KEY_EQUAL: KeyCode.Equal,
    glfw.KEY_A: KeyCode.A,
    glfw.KEY_B: KeyCode.B,
    glfw.KEY_C: KeyCode.C,
    glfw.KEY_D: KeyCode.D,
    glfw.KEY_E: KeyCode.E,
    glfw.KEY_F: KeyCode.F,
    glfw.KEY_G: KeyCode.G,
    glfw.KEY_H: KeyCode.H,
    glfw.KEY_I: KeyCode.I,
    glfw.KEY_J: KeyCode.J,
    glfw.KEY_K: KeyCode.K,
    glfw.KEY_L: KeyCode.L,
    glfw.KEY_M: KeyCode.M,
    glfw.KEY_N: KeyCode.N,
    glfw.KEY_O: KeyCode.O,
    glfw.KEY_P: KeyCode.P,
    glfw.KEY_Q: KeyCode.Q,
    glfw.KEY_R: KeyCode.R,
    glfw.KEY_S: KeyCode.S,
    glfw.KEY_T: KeyCode.T,
    glfw.KEY_U: KeyCode.U,
    glfw.KEY_V: KeyCode.V,
    glfw.KEY_W: KeyCode.W,
    glfw.KEY_X: KeyCode.X,
    glfw.KEY_Y: KeyCode.Y,
    glfw.KEY_Z: KeyCode.Z,
    glfw.KEY_LEFT_BRACKET: KeyCode.LeftBracket,
    glfw.KEY_BACKSLASH: KeyCode.Backslash,
    glfw.KEY_RIGHT_BRACKET: KeyCode.RightBracket,
    glfw.KEY_GRAVE_ACCENT: KeyCode.GraveAccent,
    glfw.KEY_ESCAPE: KeyCode.Escape,
    glfw.KEY_ENTER: KeyCode.Enter,
    glfw.KEY_TAB: KeyCode.Tab,
    glfw.KEY_BACKSPACE: KeyCode.Backspace,
    glfw.KEY_DELETE: KeyCode.Delete,
    glfw.KEY_RIGHT: KeyCode.Right,
    glfw.KEY_LEFT: KeyCode.Left,
    glfw.KEY_DOWN: KeyCode.Down,
    glfw.KEY_UP: KeyCode.Up,
    glfw.KEY_PAGE_UP: KeyCode.PageUp,
    glfw.KEY_PAGE_DOWN: KeyCode.PageDown,
    glfw.KEY_HOME: KeyCode.Home,
    glfw.KEY_END: KeyCode.End,
    glfw.KEY_CAPS_LOCK: KeyCode.CapsLock,
    glfw.KEY_F1: KeyCode.F1,
    glfw.KEY_F2: KeyCode.F2,
    glfw.KEY_F3: KeyCode.F3,
    glfw.KEY_F4: KeyCode.F4,
    glfw.KEY_F5: KeyCode.F5,
    glfw.KEY_F6: KeyCode.F6,
    glfw.KEY_F7: KeyCode.F7,
    glfw.KEY_F8: KeyCode.F8,
    glfw.KEY_F9: KeyCode.F9,
    glfw.KEY_F10: KeyCode.F10,
    glfw.KEY_F11: KeyCode.F11,
    glfw.KEY_F12: KeyCode.F12,
    glfw.KEY_KP_0: KeyCode.Keypad0,
    glfw.KEY_KP_1: KeyCode.Keypad1,
    glfw.KEY_KP_2: KeyCode.Keypad2,
    glfw.KEY_KP_3: KeyCode.Keypad3,
    glfw.KEY_KP_4: KeyCode.Keypad4,
    glfw.KEY_KP_5: KeyCode.Keypad5,
    glfw.KEY_KP_6: KeyCode.Keypad6,
    glfw.KEY_KP_7: KeyCode.Keypad7,
    glfw.KEY_KP_8: KeyCode.Keypad8,
    glfw.KEY_KP_9: KeyCode.Keypad9,
    glfw.KEY_KP_DECIMAL: KeyCode.KeypadDecimal,
    glfw.KEY_KP_DIVIDE: KeyCode.KeypadDivide,
    glfw.KEY_KP_MULTIPLY: KeyCode.KeypadMultiply,
    glfw.KEY_KP_SUBTRACT: KeyCode.KeypadSubtract,
    glfw.KEY_KP_ADD: KeyCode.KeypadAdd,
    glfw.KEY_KP_ENTER: KeyCode.KeypadEnter,
    glfw.KEY_KP_EQUAL: KeyCode.KeypadEqual,
    glfw.KEY_LEFT_SHIFT: KeyCode.LeftShift,
    glfw.KEY_LEFT_CONTROL: KeyCode.LeftControl,
    glfw.KEY_LEFT_ALT: KeyCode.LeftAlt,
    glfw.KEY_LEFT_SUPER: KeyCode.LeftSuper,
    glfw.KEY_RIGHT_SHIFT: KeyCode.RightShift,
    glfw.KEY_RIGHT_CONTROL: KeyCode.RightControl,
    glfw.KEY_RIGHT_ALT: KeyCode.RightAlt,
    glfw.KEY_RIGHT_SUPER: KeyCode.RightSuper,
    glfw.KEY_MENU: KeyCode.Menu
}

_MOUSE_MAP = {
    glfw.MOUSE_BUTTON_LEFT: MouseButton.Left,
    glfw.MOUSE_BUTTON_MIDDLE: MouseButton.Middle,
    glfw.MOUSE_BUTTON_RIGHT: MouseButton.Right
}
