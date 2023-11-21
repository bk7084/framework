use crate::core::FxHashMap;

use winit::{
    dpi::PhysicalPosition,
    event::{
        ElementState, ModifiersState, MouseButton as WinitMouseButton, MouseScrollDelta,
        VirtualKeyCode,
    },
};

#[pyo3::pyclass]
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    /// The Escape key, next to F1.
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    // TODO: rename
    Back,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,
    // also called "Next"
    NavigateForward,
    // also called "Prior"
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}

/// Convert from our KeyCode to winit's VirtualKeyCode.
impl From<VirtualKeyCode> for KeyCode {
    fn from(value: VirtualKeyCode) -> Self {
        match value {
            VirtualKeyCode::Key1 => KeyCode::Key1,
            VirtualKeyCode::Key2 => KeyCode::Key2,
            VirtualKeyCode::Key3 => KeyCode::Key3,
            VirtualKeyCode::Key4 => KeyCode::Key4,
            VirtualKeyCode::Key5 => KeyCode::Key5,
            VirtualKeyCode::Key6 => KeyCode::Key6,
            VirtualKeyCode::Key7 => KeyCode::Key7,
            VirtualKeyCode::Key8 => KeyCode::Key8,
            VirtualKeyCode::Key9 => KeyCode::Key9,
            VirtualKeyCode::Key0 => KeyCode::Key0,
            VirtualKeyCode::A => KeyCode::A,
            VirtualKeyCode::B => KeyCode::B,
            VirtualKeyCode::C => KeyCode::C,
            VirtualKeyCode::D => KeyCode::D,
            VirtualKeyCode::E => KeyCode::E,
            VirtualKeyCode::F => KeyCode::F,
            VirtualKeyCode::G => KeyCode::G,
            VirtualKeyCode::H => KeyCode::H,
            VirtualKeyCode::I => KeyCode::I,
            VirtualKeyCode::J => KeyCode::J,
            VirtualKeyCode::K => KeyCode::K,
            VirtualKeyCode::L => KeyCode::L,
            VirtualKeyCode::M => KeyCode::M,
            VirtualKeyCode::N => KeyCode::N,
            VirtualKeyCode::O => KeyCode::O,
            VirtualKeyCode::P => KeyCode::P,
            VirtualKeyCode::Q => KeyCode::Q,
            VirtualKeyCode::R => KeyCode::R,
            VirtualKeyCode::S => KeyCode::S,
            VirtualKeyCode::T => KeyCode::T,
            VirtualKeyCode::U => KeyCode::U,
            VirtualKeyCode::V => KeyCode::V,
            VirtualKeyCode::W => KeyCode::W,
            VirtualKeyCode::X => KeyCode::X,
            VirtualKeyCode::Y => KeyCode::Y,
            VirtualKeyCode::Z => KeyCode::Z,
            VirtualKeyCode::Escape => KeyCode::Escape,
            VirtualKeyCode::F1 => KeyCode::F1,
            VirtualKeyCode::F2 => KeyCode::F2,
            VirtualKeyCode::F3 => KeyCode::F3,
            VirtualKeyCode::F4 => KeyCode::F4,
            VirtualKeyCode::F5 => KeyCode::F5,
            VirtualKeyCode::F6 => KeyCode::F6,
            VirtualKeyCode::F7 => KeyCode::F7,
            VirtualKeyCode::F8 => KeyCode::F8,
            VirtualKeyCode::F9 => KeyCode::F9,
            VirtualKeyCode::F10 => KeyCode::F10,
            VirtualKeyCode::F11 => KeyCode::F11,
            VirtualKeyCode::F12 => KeyCode::F12,
            VirtualKeyCode::F13 => KeyCode::F13,
            VirtualKeyCode::F14 => KeyCode::F14,
            VirtualKeyCode::F15 => KeyCode::F15,
            VirtualKeyCode::F16 => KeyCode::F16,
            VirtualKeyCode::F17 => KeyCode::F17,
            VirtualKeyCode::F18 => KeyCode::F18,
            VirtualKeyCode::F19 => KeyCode::F19,
            VirtualKeyCode::F20 => KeyCode::F20,
            VirtualKeyCode::F21 => KeyCode::F21,
            VirtualKeyCode::F22 => KeyCode::F22,
            VirtualKeyCode::F23 => KeyCode::F23,
            VirtualKeyCode::F24 => KeyCode::F24,
            VirtualKeyCode::Snapshot => KeyCode::Snapshot,
            VirtualKeyCode::Scroll => KeyCode::Scroll,
            VirtualKeyCode::Pause => KeyCode::Pause,
            VirtualKeyCode::Insert => KeyCode::Insert,
            VirtualKeyCode::Home => KeyCode::Home,
            VirtualKeyCode::Delete => KeyCode::Delete,
            VirtualKeyCode::End => KeyCode::End,
            VirtualKeyCode::PageDown => KeyCode::PageDown,
            VirtualKeyCode::PageUp => KeyCode::PageUp,
            VirtualKeyCode::Left => KeyCode::Left,
            VirtualKeyCode::Up => KeyCode::Up,
            VirtualKeyCode::Right => KeyCode::Right,
            VirtualKeyCode::Down => KeyCode::Down,
            VirtualKeyCode::Back => KeyCode::Back,
            VirtualKeyCode::Return => KeyCode::Return,
            VirtualKeyCode::Space => KeyCode::Space,
            VirtualKeyCode::Compose => KeyCode::Compose,
            VirtualKeyCode::Caret => KeyCode::Caret,
            VirtualKeyCode::Numlock => KeyCode::Numlock,
            VirtualKeyCode::Numpad0 => KeyCode::Numpad0,
            VirtualKeyCode::Numpad1 => KeyCode::Numpad1,
            VirtualKeyCode::Numpad2 => KeyCode::Numpad2,
            VirtualKeyCode::Numpad3 => KeyCode::Numpad3,
            VirtualKeyCode::Numpad4 => KeyCode::Numpad4,
            VirtualKeyCode::Numpad5 => KeyCode::Numpad5,
            VirtualKeyCode::Numpad6 => KeyCode::Numpad6,
            VirtualKeyCode::Numpad7 => KeyCode::Numpad7,
            VirtualKeyCode::Numpad8 => KeyCode::Numpad8,
            VirtualKeyCode::Numpad9 => KeyCode::Numpad9,
            VirtualKeyCode::AbntC1 => KeyCode::AbntC1,
            VirtualKeyCode::NumpadAdd => KeyCode::NumpadAdd,
            VirtualKeyCode::NumpadDivide => KeyCode::NumpadDivide,
            VirtualKeyCode::NumpadDecimal => KeyCode::NumpadDecimal,
            VirtualKeyCode::NumpadComma => KeyCode::NumpadComma,
            VirtualKeyCode::NumpadEnter => KeyCode::NumpadEnter,
            VirtualKeyCode::NumpadEquals => KeyCode::NumpadEquals,
            VirtualKeyCode::NumpadMultiply => KeyCode::NumpadMultiply,
            VirtualKeyCode::NumpadSubtract => KeyCode::NumpadSubtract,
            VirtualKeyCode::AbntC2 => KeyCode::AbntC2,
            VirtualKeyCode::Apostrophe => KeyCode::Apostrophe,
            VirtualKeyCode::Apps => KeyCode::Apps,
            VirtualKeyCode::Asterisk => KeyCode::Asterisk,
            VirtualKeyCode::At => KeyCode::At,
            VirtualKeyCode::Ax => KeyCode::Ax,
            VirtualKeyCode::Backslash => KeyCode::Backslash,
            VirtualKeyCode::Calculator => KeyCode::Calculator,
            VirtualKeyCode::Capital => KeyCode::Capital,
            VirtualKeyCode::Colon => KeyCode::Colon,
            VirtualKeyCode::Comma => KeyCode::Comma,
            VirtualKeyCode::Convert => KeyCode::Convert,
            VirtualKeyCode::Equals => KeyCode::Equals,
            VirtualKeyCode::Grave => KeyCode::Grave,
            VirtualKeyCode::Kana => KeyCode::Kana,
            VirtualKeyCode::Kanji => KeyCode::Kanji,
            VirtualKeyCode::LAlt => KeyCode::LAlt,
            VirtualKeyCode::LBracket => KeyCode::LBracket,
            VirtualKeyCode::LControl => KeyCode::LControl,
            VirtualKeyCode::LShift => KeyCode::LShift,
            VirtualKeyCode::LWin => KeyCode::LWin,
            VirtualKeyCode::Mail => KeyCode::Mail,
            VirtualKeyCode::MediaSelect => KeyCode::MediaSelect,
            VirtualKeyCode::MediaStop => KeyCode::MediaStop,
            VirtualKeyCode::Minus => KeyCode::Minus,
            VirtualKeyCode::Mute => KeyCode::Mute,
            VirtualKeyCode::MyComputer => KeyCode::MyComputer,
            VirtualKeyCode::NavigateForward => KeyCode::NavigateForward,
            VirtualKeyCode::NavigateBackward => KeyCode::NavigateBackward,
            VirtualKeyCode::NextTrack => KeyCode::NextTrack,
            VirtualKeyCode::NoConvert => KeyCode::NoConvert,
            VirtualKeyCode::OEM102 => KeyCode::OEM102,
            VirtualKeyCode::Period => KeyCode::Period,
            VirtualKeyCode::PlayPause => KeyCode::PlayPause,
            VirtualKeyCode::Plus => KeyCode::Plus,
            VirtualKeyCode::Power => KeyCode::Power,
            VirtualKeyCode::PrevTrack => KeyCode::PrevTrack,
            VirtualKeyCode::RAlt => KeyCode::RAlt,
            VirtualKeyCode::RBracket => KeyCode::RBracket,
            VirtualKeyCode::RControl => KeyCode::RControl,
            VirtualKeyCode::RShift => KeyCode::RShift,
            VirtualKeyCode::RWin => KeyCode::RWin,
            VirtualKeyCode::Semicolon => KeyCode::Semicolon,
            VirtualKeyCode::Slash => KeyCode::Slash,
            VirtualKeyCode::Sleep => KeyCode::Sleep,
            VirtualKeyCode::Stop => KeyCode::Stop,
            VirtualKeyCode::Sysrq => KeyCode::Sysrq,
            VirtualKeyCode::Tab => KeyCode::Tab,
            VirtualKeyCode::Underline => KeyCode::Underline,
            VirtualKeyCode::Unlabeled => KeyCode::Unlabeled,
            VirtualKeyCode::VolumeDown => KeyCode::VolumeDown,
            VirtualKeyCode::VolumeUp => KeyCode::VolumeUp,
            VirtualKeyCode::Wake => KeyCode::Wake,
            VirtualKeyCode::WebBack => KeyCode::WebBack,
            VirtualKeyCode::WebFavorites => KeyCode::WebFavorites,
            VirtualKeyCode::WebForward => KeyCode::WebForward,
            VirtualKeyCode::WebHome => KeyCode::WebHome,
            VirtualKeyCode::WebRefresh => KeyCode::WebRefresh,
            VirtualKeyCode::WebSearch => KeyCode::WebSearch,
            VirtualKeyCode::WebStop => KeyCode::WebStop,
            VirtualKeyCode::Yen => KeyCode::Yen,
            VirtualKeyCode::Copy => KeyCode::Copy,
            VirtualKeyCode::Paste => KeyCode::Paste,
            VirtualKeyCode::Cut => KeyCode::Cut,
        }
    }
}

/// Convert from winit's VirtualKeyCode to our KeyCode.
impl From<KeyCode> for VirtualKeyCode {
    fn from(value: KeyCode) -> Self {
        match value {
            KeyCode::Key1 => VirtualKeyCode::Key1,
            KeyCode::Key2 => VirtualKeyCode::Key2,
            KeyCode::Key3 => VirtualKeyCode::Key3,
            KeyCode::Key4 => VirtualKeyCode::Key4,
            KeyCode::Key5 => VirtualKeyCode::Key5,
            KeyCode::Key6 => VirtualKeyCode::Key6,
            KeyCode::Key7 => VirtualKeyCode::Key7,
            KeyCode::Key8 => VirtualKeyCode::Key8,
            KeyCode::Key9 => VirtualKeyCode::Key9,
            KeyCode::Key0 => VirtualKeyCode::Key0,
            KeyCode::A => VirtualKeyCode::A,
            KeyCode::B => VirtualKeyCode::B,
            KeyCode::C => VirtualKeyCode::C,
            KeyCode::D => VirtualKeyCode::D,
            KeyCode::E => VirtualKeyCode::E,
            KeyCode::F => VirtualKeyCode::F,
            KeyCode::G => VirtualKeyCode::G,
            KeyCode::H => VirtualKeyCode::H,
            KeyCode::I => VirtualKeyCode::I,
            KeyCode::J => VirtualKeyCode::J,
            KeyCode::K => VirtualKeyCode::K,
            KeyCode::L => VirtualKeyCode::L,
            KeyCode::M => VirtualKeyCode::M,
            KeyCode::N => VirtualKeyCode::N,
            KeyCode::O => VirtualKeyCode::O,
            KeyCode::P => VirtualKeyCode::P,
            KeyCode::Q => VirtualKeyCode::Q,
            KeyCode::R => VirtualKeyCode::R,
            KeyCode::S => VirtualKeyCode::S,
            KeyCode::T => VirtualKeyCode::T,
            KeyCode::U => VirtualKeyCode::U,
            KeyCode::V => VirtualKeyCode::V,
            KeyCode::W => VirtualKeyCode::W,
            KeyCode::X => VirtualKeyCode::X,
            KeyCode::Y => VirtualKeyCode::Y,
            KeyCode::Z => VirtualKeyCode::Z,
            KeyCode::Escape => VirtualKeyCode::Escape,
            KeyCode::F1 => VirtualKeyCode::F1,
            KeyCode::F2 => VirtualKeyCode::F2,
            KeyCode::F3 => VirtualKeyCode::F3,
            KeyCode::F4 => VirtualKeyCode::F4,
            KeyCode::F5 => VirtualKeyCode::F5,
            KeyCode::F6 => VirtualKeyCode::F6,
            KeyCode::F7 => VirtualKeyCode::F7,
            KeyCode::F8 => VirtualKeyCode::F8,
            KeyCode::F9 => VirtualKeyCode::F9,
            KeyCode::F10 => VirtualKeyCode::F10,
            KeyCode::F11 => VirtualKeyCode::F11,
            KeyCode::F12 => VirtualKeyCode::F12,
            KeyCode::F13 => VirtualKeyCode::F13,
            KeyCode::F14 => VirtualKeyCode::F14,
            KeyCode::F15 => VirtualKeyCode::F15,
            KeyCode::F16 => VirtualKeyCode::F16,
            KeyCode::F17 => VirtualKeyCode::F17,
            KeyCode::F18 => VirtualKeyCode::F18,
            KeyCode::F19 => VirtualKeyCode::F19,
            KeyCode::F20 => VirtualKeyCode::F20,
            KeyCode::F21 => VirtualKeyCode::F21,
            KeyCode::F22 => VirtualKeyCode::F22,
            KeyCode::F23 => VirtualKeyCode::F23,
            KeyCode::F24 => VirtualKeyCode::F24,
            KeyCode::Snapshot => VirtualKeyCode::Snapshot,
            KeyCode::Scroll => VirtualKeyCode::Scroll,
            KeyCode::Pause => VirtualKeyCode::Pause,
            KeyCode::Insert => VirtualKeyCode::Insert,
            KeyCode::Home => VirtualKeyCode::Home,
            KeyCode::Delete => VirtualKeyCode::Delete,
            KeyCode::End => VirtualKeyCode::End,
            KeyCode::PageDown => VirtualKeyCode::PageDown,
            KeyCode::PageUp => VirtualKeyCode::PageUp,
            KeyCode::Left => VirtualKeyCode::Left,
            KeyCode::Up => VirtualKeyCode::Up,
            KeyCode::Right => VirtualKeyCode::Right,
            KeyCode::Down => VirtualKeyCode::Down,
            KeyCode::Back => VirtualKeyCode::Back,
            KeyCode::Return => VirtualKeyCode::Return,
            KeyCode::Space => VirtualKeyCode::Space,
            KeyCode::Compose => VirtualKeyCode::Compose,
            KeyCode::Caret => VirtualKeyCode::Caret,
            KeyCode::Numlock => VirtualKeyCode::Numlock,
            KeyCode::Numpad0 => VirtualKeyCode::Numpad0,
            KeyCode::Numpad1 => VirtualKeyCode::Numpad1,
            KeyCode::Numpad2 => VirtualKeyCode::Numpad2,
            KeyCode::Numpad3 => VirtualKeyCode::Numpad3,
            KeyCode::Numpad4 => VirtualKeyCode::Numpad4,
            KeyCode::Numpad5 => VirtualKeyCode::Numpad5,
            KeyCode::Numpad6 => VirtualKeyCode::Numpad6,
            KeyCode::Numpad7 => VirtualKeyCode::Numpad7,
            KeyCode::Numpad8 => VirtualKeyCode::Numpad8,
            KeyCode::Numpad9 => VirtualKeyCode::Numpad9,
            KeyCode::AbntC1 => VirtualKeyCode::AbntC1,
            KeyCode::AbntC2 => VirtualKeyCode::AbntC2,
            KeyCode::Apostrophe => VirtualKeyCode::Apostrophe,
            KeyCode::Apps => VirtualKeyCode::Apps,
            KeyCode::Asterisk => VirtualKeyCode::Asterisk,
            KeyCode::At => VirtualKeyCode::At,
            KeyCode::Ax => VirtualKeyCode::Ax,
            KeyCode::Backslash => VirtualKeyCode::Backslash,
            KeyCode::Calculator => VirtualKeyCode::Calculator,
            KeyCode::Capital => VirtualKeyCode::Capital,
            KeyCode::Colon => VirtualKeyCode::Colon,
            KeyCode::Comma => VirtualKeyCode::Comma,
            KeyCode::Convert => VirtualKeyCode::Convert,
            KeyCode::Equals => VirtualKeyCode::Equals,
            KeyCode::Grave => VirtualKeyCode::Grave,
            KeyCode::Kana => VirtualKeyCode::Kana,
            KeyCode::Kanji => VirtualKeyCode::Kanji,
            KeyCode::LAlt => VirtualKeyCode::LAlt,
            KeyCode::LBracket => VirtualKeyCode::LBracket,
            KeyCode::LControl => VirtualKeyCode::LControl,
            KeyCode::LShift => VirtualKeyCode::LShift,
            KeyCode::LWin => VirtualKeyCode::LWin,
            KeyCode::Mail => VirtualKeyCode::Mail,
            KeyCode::MediaSelect => VirtualKeyCode::MediaSelect,
            KeyCode::MediaStop => VirtualKeyCode::MediaStop,
            KeyCode::Minus => VirtualKeyCode::Minus,
            KeyCode::Mute => VirtualKeyCode::Mute,
            KeyCode::MyComputer => VirtualKeyCode::MyComputer,
            KeyCode::NavigateForward => VirtualKeyCode::NavigateForward,
            KeyCode::NavigateBackward => VirtualKeyCode::NavigateBackward,
            KeyCode::NextTrack => VirtualKeyCode::NextTrack,
            KeyCode::NoConvert => VirtualKeyCode::NoConvert,
            KeyCode::NumpadComma => VirtualKeyCode::NumpadComma,
            KeyCode::NumpadEnter => VirtualKeyCode::NumpadEnter,
            KeyCode::NumpadEquals => VirtualKeyCode::NumpadEquals,
            KeyCode::OEM102 => VirtualKeyCode::OEM102,
            KeyCode::Period => VirtualKeyCode::Period,
            KeyCode::PlayPause => VirtualKeyCode::PlayPause,
            KeyCode::Power => VirtualKeyCode::Power,
            KeyCode::PrevTrack => VirtualKeyCode::PrevTrack,
            KeyCode::RAlt => VirtualKeyCode::RAlt,
            KeyCode::RBracket => VirtualKeyCode::RBracket,
            KeyCode::RControl => VirtualKeyCode::RControl,
            KeyCode::RShift => VirtualKeyCode::RShift,
            KeyCode::RWin => VirtualKeyCode::RWin,
            KeyCode::Semicolon => VirtualKeyCode::Semicolon,
            KeyCode::Slash => VirtualKeyCode::Slash,
            KeyCode::Sleep => VirtualKeyCode::Sleep,
            KeyCode::Stop => VirtualKeyCode::Stop,
            KeyCode::Sysrq => VirtualKeyCode::Sysrq,
            KeyCode::Tab => VirtualKeyCode::Tab,
            KeyCode::Underline => VirtualKeyCode::Underline,
            KeyCode::Unlabeled => VirtualKeyCode::Unlabeled,
            KeyCode::VolumeDown => VirtualKeyCode::VolumeDown,
            KeyCode::VolumeUp => VirtualKeyCode::VolumeUp,
            KeyCode::Wake => VirtualKeyCode::Wake,
            KeyCode::WebBack => VirtualKeyCode::WebBack,
            KeyCode::WebFavorites => VirtualKeyCode::WebFavorites,
            KeyCode::WebForward => VirtualKeyCode::WebForward,
            KeyCode::WebHome => VirtualKeyCode::WebHome,
            KeyCode::WebRefresh => VirtualKeyCode::WebRefresh,
            KeyCode::WebSearch => VirtualKeyCode::WebSearch,
            KeyCode::WebStop => VirtualKeyCode::WebStop,
            KeyCode::Yen => VirtualKeyCode::Yen,
            KeyCode::Copy => VirtualKeyCode::Copy,
            KeyCode::Paste => VirtualKeyCode::Paste,
            KeyCode::Cut => VirtualKeyCode::Cut,
            KeyCode::NumpadAdd => VirtualKeyCode::NumpadAdd,
            KeyCode::NumpadDivide => VirtualKeyCode::NumpadDivide,
            KeyCode::NumpadDecimal => VirtualKeyCode::NumpadDecimal,
            KeyCode::NumpadMultiply => VirtualKeyCode::NumpadMultiply,
            KeyCode::NumpadSubtract => VirtualKeyCode::NumpadSubtract,
            KeyCode::Plus => VirtualKeyCode::Plus,
        }
    }
}

#[pyo3::pyclass]
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl From<MouseButton> for WinitMouseButton {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Left => WinitMouseButton::Left,
            MouseButton::Right => WinitMouseButton::Right,
            MouseButton::Middle => WinitMouseButton::Middle,
        }
    }
}

/// Struct holding the state of the keyboard and mouse.
#[derive(Debug, Clone)]
pub struct InputState {
    pub keys: FxHashMap<VirtualKeyCode, bool>,
    pub btns: FxHashMap<WinitMouseButton, bool>,
    pub mods: ModifiersState,
    pub scroll_delta: f32,
    pub cursor_delta: [f32; 2],
    pub cursor_pos: [f32; 2],
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            keys: Default::default(),
            btns: Default::default(),
            mods: Default::default(),
            scroll_delta: 0.0,
            cursor_delta: [0.0, 0.0],
            cursor_pos: [0.0, 0.0],
        }
    }
}

/// Python interface for the InputState struct.
impl InputState {
    pub fn is_key_pressed(&self, key_code: KeyCode) -> bool {
        let key_code = VirtualKeyCode::from(key_code);
        *self.keys.get(&key_code).unwrap_or(&false)
    }

    pub fn is_key_released(&self, key_code: KeyCode) -> bool {
        !self.is_key_pressed(key_code)
    }

    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        *self.btns.get(&button.into()).unwrap_or(&false)
    }

    pub fn is_mouse_released(&self, button: MouseButton) -> bool {
        !self.is_mouse_pressed(button)
    }
}

impl InputState {
    pub fn take(&mut self) -> Input {
        let mut input = Input {
            keys: [None; 16],
            btns: 0,
            scroll_delta: self.scroll_delta,
            cursor_delta: self.cursor_delta,
            cursor_pos: self.cursor_pos,
        };
        let mut i = 0;
        self.keys.iter().for_each(|(k, v)| {
            if i < input.keys.len() && *v {
                input.keys[i] = Some(KeyCode::from(*k));
                i += 1;
            }
        });
        if *self.btns.get(&WinitMouseButton::Left).unwrap_or(&false) {
            input.btns = 1 << 0;
        }
        if *self.btns.get(&WinitMouseButton::Right).unwrap_or(&false) {
            input.btns |= 1 << 1;
        }
        if *self.btns.get(&WinitMouseButton::Middle).unwrap_or(&false) {
            input.btns |= 1 << 2;
        }
        self.cursor_delta = [0.0, 0.0];
        self.scroll_delta = 0.0;
        input
    }

    pub fn update_key_states(&mut self, key_code: VirtualKeyCode, state: ElementState) {
        log::trace!("update_key_states: {:?} {:?}", key_code, state);
        *self.keys.entry(key_code).or_insert(false) = state == ElementState::Pressed;
    }

    pub fn update_mouse_button_states(&mut self, button: WinitMouseButton, state: ElementState) {
        log::trace!("update_mouse_button_states: {:?} {:?}", button, state);
        *self.btns.entry(button).or_insert(false) = state == ElementState::Pressed;
    }

    pub fn update_modifier_states(&mut self, modifier_state: ModifiersState) {
        log::trace!("update_modifier_states: {:?}", modifier_state);
        self.mods = modifier_state;
    }

    pub fn update_cursor_delta(&mut self, new_pos: PhysicalPosition<f64>) {
        log::trace!("update_cursor_delta: {:?}", new_pos);
        self.cursor_delta = [
            new_pos.x as f32 - self.cursor_pos[0],
            new_pos.y as f32 - self.cursor_pos[1],
        ];
        self.cursor_pos = new_pos.into();
    }

    pub fn update_scroll_delta(&mut self, delta: MouseScrollDelta) {
        log::trace!("update_scroll_delta: {:?}", delta);
        self.scroll_delta = match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                -y * 100.0 // assuming a line is about 100 pixels
            }
            MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
        };
    }
}

/// Struct holding the input state of the current frame.
/// This is passed to the user's update function.
#[pyo3::pyclass]
#[derive(Debug, Copy, Clone)]
pub struct Input {
    /// The keys that were pressed this frame, 8 at most.
    keys: [Option<KeyCode>; 16],
    /// The mouse buttons that were pressed this frame.
    btns: u32,
    /// The scroll delta of the mouse wheel.
    scroll_delta: f32,
    /// The delta of the cursor position since the last frame.
    cursor_delta: [f32; 2],
    /// The current cursor position.
    cursor_pos: [f32; 2],
}

static_assertions::assert_eq_size!(Input, [u32; 22]);

#[pyo3::pymethods]
impl Input {
    #[getter]
    pub fn cursor_position(&self) -> [f32; 2] {
        self.cursor_pos
    }

    #[getter]
    pub fn cursor_delta(&self) -> [f32; 2] {
        self.cursor_delta
    }

    #[getter]
    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    pub fn is_shift_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::LShift) || self.is_key_pressed(KeyCode::RShift)
    }

    pub fn is_left_shift_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::LShift)
    }

    pub fn is_right_shift_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::RShift)
    }

    pub fn is_ctrl_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::LControl) || self.is_key_pressed(KeyCode::RControl)
    }

    pub fn is_left_ctrl_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::LControl)
    }

    pub fn is_right_ctrl_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::RControl)
    }

    pub fn is_alt_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::LAlt) || self.is_key_pressed(KeyCode::RAlt)
    }

    pub fn is_left_alt_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::LAlt)
    }

    pub fn is_right_alt_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::RAlt)
    }

    pub fn is_super_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::LWin) || self.is_key_pressed(KeyCode::RWin)
    }

    pub fn is_key_pressed(&self, key_code: KeyCode) -> bool {
        self.keys.iter().any(|k| *k == Some(key_code))
    }

    pub fn is_key_released(&self, key_code: KeyCode) -> bool {
        !self.is_key_pressed(key_code)
    }

    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.btns & (1 << button as u32) != 0
    }

    pub fn is_mouse_released(&self, button: MouseButton) -> bool {
        !self.is_mouse_pressed(button)
    }

    pub fn release_key(&mut self, key_code: KeyCode) {
        self.keys.iter_mut().for_each(|k| {
            if *k == Some(key_code) {
                *k = None;
            }
        });
    }

    pub fn release_mouse_button(&mut self, button: MouseButton) {
        self.btns &= !(1 << button as u32);
    }
}
