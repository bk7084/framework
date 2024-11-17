use crate::core::FxHashMap;

use winit::event::Modifiers;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta},
    keyboard::{KeyCode as WinitKeyCode, ModifiersState},
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

    PrintScreen,
    ScrollLock,
    Pause,

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

    Backspace,
    Backquote,
    Enter,
    Space,
    NumLock,
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
    NumpadEqual,
    NumpadMultiply,
    NumpadSubtract,
    NumpadBackspace,
    NumpadClear,
    NumpadClearEntry,
    NumpadHash,
    NumpadMemoryAdd,
    NumpadMemoryClear,
    NumpadMemoryRecall,
    NumpadMemoryStore,
    NumpadMemorySubtract,
    NumpadParenLeft,
    NumpadParenRight,
    NumpadStar,

    Backslash,
    CapsLock,
    Comma,
    Convert,
    Equal,
    AltLeft,
    BracketLeft,
    ControlLeft,
    ShiftLeft,
    SuperLeft,
    SuperRight,
    LaunchMail,
    MediaSelect,
    MediaStop,
    Minus,
    AudioVolumeMute,
    MediaTrackNext,
    NonConvert,
    Period,
    MediaPlayPause,
    Power,
    MediaTrackPrevious,
    AltRight,
    BracketRight,
    ControlRight,
    ShiftRight,
    Semicolon,
    Slash,
    Sleep,
    Tab,
    AudioVolumeDown,
    AudioVolumeUp,
    WakeUp,
    BrowserBack,
    BrowserFavorites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    Copy,
    Paste,
    Cut,

    IntlBackslash,
    IntlRo,
    IntlYen,
    Quote,

    ContextMenu,
    KanaMode,
    Lang1,
    Lang2,
    Lang3,
    Lang4,
    Lang5,

    Help,
    Fn,
    FnLock,
    Eject,

    LaunchApp1,
    LaunchApp2,

    Meta,
    Hyper,
    Turbo,
    Abort,
    Resume,
    Suspend,
    Again,
    Find,
    Open,
    Props,
    Select,
    Undo,
    Hiragana,
    Katakana,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
}

/// Convert from our KeyCode to winit KeyCode.
impl From<WinitKeyCode> for KeyCode {
    fn from(value: WinitKeyCode) -> Self {
        match value {
            WinitKeyCode::Digit1 => KeyCode::Key1,
            WinitKeyCode::Digit2 => KeyCode::Key2,
            WinitKeyCode::Digit3 => KeyCode::Key3,
            WinitKeyCode::Digit4 => KeyCode::Key4,
            WinitKeyCode::Digit5 => KeyCode::Key5,
            WinitKeyCode::Digit6 => KeyCode::Key6,
            WinitKeyCode::Digit7 => KeyCode::Key7,
            WinitKeyCode::Digit8 => KeyCode::Key8,
            WinitKeyCode::Digit9 => KeyCode::Key9,
            WinitKeyCode::Digit0 => KeyCode::Key0,
            WinitKeyCode::KeyA => KeyCode::A,
            WinitKeyCode::KeyB => KeyCode::B,
            WinitKeyCode::KeyC => KeyCode::C,
            WinitKeyCode::KeyD => KeyCode::D,
            WinitKeyCode::KeyE => KeyCode::E,
            WinitKeyCode::KeyF => KeyCode::F,
            WinitKeyCode::KeyG => KeyCode::G,
            WinitKeyCode::KeyH => KeyCode::H,
            WinitKeyCode::KeyI => KeyCode::I,
            WinitKeyCode::KeyJ => KeyCode::J,
            WinitKeyCode::KeyK => KeyCode::K,
            WinitKeyCode::KeyL => KeyCode::L,
            WinitKeyCode::KeyM => KeyCode::M,
            WinitKeyCode::KeyN => KeyCode::N,
            WinitKeyCode::KeyO => KeyCode::O,
            WinitKeyCode::KeyP => KeyCode::P,
            WinitKeyCode::KeyQ => KeyCode::Q,
            WinitKeyCode::KeyR => KeyCode::R,
            WinitKeyCode::KeyS => KeyCode::S,
            WinitKeyCode::KeyT => KeyCode::T,
            WinitKeyCode::KeyU => KeyCode::U,
            WinitKeyCode::KeyV => KeyCode::V,
            WinitKeyCode::KeyW => KeyCode::W,
            WinitKeyCode::KeyX => KeyCode::X,
            WinitKeyCode::KeyY => KeyCode::Y,
            WinitKeyCode::KeyZ => KeyCode::Z,
            WinitKeyCode::Escape => KeyCode::Escape,
            WinitKeyCode::F1 => KeyCode::F1,
            WinitKeyCode::F2 => KeyCode::F2,
            WinitKeyCode::F3 => KeyCode::F3,
            WinitKeyCode::F4 => KeyCode::F4,
            WinitKeyCode::F5 => KeyCode::F5,
            WinitKeyCode::F6 => KeyCode::F6,
            WinitKeyCode::F7 => KeyCode::F7,
            WinitKeyCode::F8 => KeyCode::F8,
            WinitKeyCode::F9 => KeyCode::F9,
            WinitKeyCode::F10 => KeyCode::F10,
            WinitKeyCode::F11 => KeyCode::F11,
            WinitKeyCode::F12 => KeyCode::F12,
            WinitKeyCode::F13 => KeyCode::F13,
            WinitKeyCode::F14 => KeyCode::F14,
            WinitKeyCode::F15 => KeyCode::F15,
            WinitKeyCode::F16 => KeyCode::F16,
            WinitKeyCode::F17 => KeyCode::F17,
            WinitKeyCode::F18 => KeyCode::F18,
            WinitKeyCode::F19 => KeyCode::F19,
            WinitKeyCode::F20 => KeyCode::F20,
            WinitKeyCode::F21 => KeyCode::F21,
            WinitKeyCode::F22 => KeyCode::F22,
            WinitKeyCode::F23 => KeyCode::F23,
            WinitKeyCode::F24 => KeyCode::F24,
            WinitKeyCode::PrintScreen => KeyCode::PrintScreen,
            WinitKeyCode::ScrollLock => KeyCode::ScrollLock,
            WinitKeyCode::Pause => KeyCode::Pause,
            WinitKeyCode::Insert => KeyCode::Insert,
            WinitKeyCode::Home => KeyCode::Home,
            WinitKeyCode::Delete => KeyCode::Delete,
            WinitKeyCode::End => KeyCode::End,
            WinitKeyCode::PageDown => KeyCode::PageDown,
            WinitKeyCode::PageUp => KeyCode::PageUp,
            WinitKeyCode::ArrowLeft => KeyCode::Left,
            WinitKeyCode::ArrowUp => KeyCode::Up,
            WinitKeyCode::ArrowRight => KeyCode::Right,
            WinitKeyCode::ArrowDown => KeyCode::Down,
            WinitKeyCode::Backspace => KeyCode::Backspace,
            WinitKeyCode::Enter => KeyCode::Enter,
            WinitKeyCode::Space => KeyCode::Space,
            WinitKeyCode::NumLock => KeyCode::NumLock,
            WinitKeyCode::Numpad0 => KeyCode::Numpad0,
            WinitKeyCode::Numpad1 => KeyCode::Numpad1,
            WinitKeyCode::Numpad2 => KeyCode::Numpad2,
            WinitKeyCode::Numpad3 => KeyCode::Numpad3,
            WinitKeyCode::Numpad4 => KeyCode::Numpad4,
            WinitKeyCode::Numpad5 => KeyCode::Numpad5,
            WinitKeyCode::Numpad6 => KeyCode::Numpad6,
            WinitKeyCode::Numpad7 => KeyCode::Numpad7,
            WinitKeyCode::Numpad8 => KeyCode::Numpad8,
            WinitKeyCode::Numpad9 => KeyCode::Numpad9,
            WinitKeyCode::NumpadAdd => KeyCode::NumpadAdd,
            WinitKeyCode::NumpadDivide => KeyCode::NumpadDivide,
            WinitKeyCode::NumpadDecimal => KeyCode::NumpadDecimal,
            WinitKeyCode::NumpadComma => KeyCode::NumpadComma,
            WinitKeyCode::NumpadEnter => KeyCode::NumpadEnter,
            WinitKeyCode::NumpadEqual => KeyCode::NumpadEqual,
            WinitKeyCode::NumpadMultiply => KeyCode::NumpadMultiply,
            WinitKeyCode::NumpadSubtract => KeyCode::NumpadSubtract,
            WinitKeyCode::Backslash => KeyCode::Backslash,
            WinitKeyCode::CapsLock => KeyCode::CapsLock,
            WinitKeyCode::Comma => KeyCode::Comma,
            WinitKeyCode::Convert => KeyCode::Convert,
            WinitKeyCode::Equal => KeyCode::Equal,
            WinitKeyCode::AltLeft => KeyCode::AltLeft,
            WinitKeyCode::BracketLeft => KeyCode::BracketLeft,
            WinitKeyCode::ControlLeft => KeyCode::ControlLeft,
            WinitKeyCode::ShiftLeft => KeyCode::ShiftLeft,
            WinitKeyCode::SuperLeft => KeyCode::SuperLeft,
            WinitKeyCode::LaunchMail => KeyCode::LaunchMail,
            WinitKeyCode::MediaSelect => KeyCode::MediaSelect,
            WinitKeyCode::MediaStop => KeyCode::MediaStop,
            WinitKeyCode::Minus => KeyCode::Minus,
            WinitKeyCode::AudioVolumeMute => KeyCode::AudioVolumeMute,
            WinitKeyCode::MediaTrackNext => KeyCode::MediaTrackNext,
            WinitKeyCode::Period => KeyCode::Period,
            WinitKeyCode::MediaTrackPrevious => KeyCode::MediaPlayPause,
            WinitKeyCode::Power => KeyCode::Power,
            WinitKeyCode::Backquote => KeyCode::Backquote,
            WinitKeyCode::BracketRight => KeyCode::BracketRight,
            WinitKeyCode::IntlBackslash => KeyCode::IntlBackslash,
            WinitKeyCode::IntlRo => KeyCode::IntlRo,
            WinitKeyCode::IntlYen => KeyCode::IntlYen,
            WinitKeyCode::Quote => KeyCode::Quote,
            WinitKeyCode::Semicolon => KeyCode::Semicolon,
            WinitKeyCode::Slash => KeyCode::Slash,
            WinitKeyCode::AltRight => KeyCode::AltRight,
            WinitKeyCode::ContextMenu => KeyCode::ContextMenu,
            WinitKeyCode::ControlRight => KeyCode::ControlRight,
            WinitKeyCode::SuperRight => KeyCode::SuperRight,
            WinitKeyCode::ShiftRight => KeyCode::ShiftRight,
            WinitKeyCode::Tab => KeyCode::Tab,
            WinitKeyCode::KanaMode => KeyCode::KanaMode,
            WinitKeyCode::Lang1 => KeyCode::Lang1,
            WinitKeyCode::Lang2 => KeyCode::Lang2,
            WinitKeyCode::Lang3 => KeyCode::Lang3,
            WinitKeyCode::Lang4 => KeyCode::Lang4,
            WinitKeyCode::Lang5 => KeyCode::Lang5,
            WinitKeyCode::NonConvert => KeyCode::NonConvert,
            WinitKeyCode::Help => KeyCode::Help,
            WinitKeyCode::NumpadBackspace => KeyCode::NumpadBackspace,
            WinitKeyCode::NumpadClear => KeyCode::NumpadClear,
            WinitKeyCode::NumpadClearEntry => KeyCode::NumpadClearEntry,
            WinitKeyCode::NumpadHash => KeyCode::NumpadHash,
            WinitKeyCode::NumpadMemoryAdd => KeyCode::NumpadMemoryAdd,
            WinitKeyCode::NumpadMemoryClear => KeyCode::NumpadMemoryClear,
            WinitKeyCode::NumpadMemoryRecall => KeyCode::NumpadMemoryRecall,
            WinitKeyCode::NumpadMemoryStore => KeyCode::NumpadMemoryStore,
            WinitKeyCode::NumpadMemorySubtract => KeyCode::NumpadMemorySubtract,
            WinitKeyCode::NumpadParenLeft => KeyCode::NumpadParenLeft,
            WinitKeyCode::NumpadParenRight => KeyCode::NumpadParenRight,
            WinitKeyCode::NumpadStar => KeyCode::NumpadStar,
            WinitKeyCode::Fn => KeyCode::Fn,
            WinitKeyCode::FnLock => KeyCode::FnLock,
            WinitKeyCode::BrowserBack => KeyCode::BrowserBack,
            WinitKeyCode::BrowserFavorites => KeyCode::BrowserFavorites,
            WinitKeyCode::BrowserForward => KeyCode::BrowserForward,
            WinitKeyCode::BrowserHome => KeyCode::BrowserHome,
            WinitKeyCode::BrowserRefresh => KeyCode::BrowserRefresh,
            WinitKeyCode::BrowserSearch => KeyCode::BrowserSearch,
            WinitKeyCode::BrowserStop => KeyCode::BrowserStop,
            WinitKeyCode::Eject => KeyCode::Eject,
            WinitKeyCode::LaunchApp1 => KeyCode::LaunchApp1,
            WinitKeyCode::LaunchApp2 => KeyCode::LaunchApp2,
            WinitKeyCode::MediaPlayPause => KeyCode::MediaPlayPause,
            WinitKeyCode::Sleep => KeyCode::Sleep,
            WinitKeyCode::AudioVolumeDown => KeyCode::AudioVolumeDown,
            WinitKeyCode::AudioVolumeUp => KeyCode::AudioVolumeUp,
            WinitKeyCode::WakeUp => KeyCode::WakeUp,
            WinitKeyCode::Meta => KeyCode::Meta,
            WinitKeyCode::Hyper => KeyCode::Hyper,
            WinitKeyCode::Turbo => KeyCode::Turbo,
            WinitKeyCode::Abort => KeyCode::Abort,
            WinitKeyCode::Resume => KeyCode::Resume,
            WinitKeyCode::Suspend => KeyCode::Suspend,
            WinitKeyCode::Again => KeyCode::Again,
            WinitKeyCode::Copy => KeyCode::Copy,
            WinitKeyCode::Cut => KeyCode::Cut,
            WinitKeyCode::Find => KeyCode::Find,
            WinitKeyCode::Open => KeyCode::Open,
            WinitKeyCode::Paste => KeyCode::Paste,
            WinitKeyCode::Props => KeyCode::Props,
            WinitKeyCode::Select => KeyCode::Select,
            WinitKeyCode::Undo => KeyCode::Undo,
            WinitKeyCode::Hiragana => KeyCode::Hiragana,
            WinitKeyCode::Katakana => KeyCode::Katakana,
            WinitKeyCode::F25 => KeyCode::F25,
            WinitKeyCode::F26 => KeyCode::F26,
            WinitKeyCode::F27 => KeyCode::F27,
            WinitKeyCode::F28 => KeyCode::F28,
            WinitKeyCode::F29 => KeyCode::F29,
            WinitKeyCode::F30 => KeyCode::F30,
            WinitKeyCode::F31 => KeyCode::F31,
            WinitKeyCode::F32 => KeyCode::F32,
            WinitKeyCode::F33 => KeyCode::F33,
            WinitKeyCode::F34 => KeyCode::F34,
            WinitKeyCode::F35 => KeyCode::F35,
            _ => todo!("Implement missing KeyCode: {:?}", value),
        }
    }
}

/// Convert from winit KeyCode to our KeyCode.
impl From<KeyCode> for WinitKeyCode {
    fn from(value: KeyCode) -> Self {
        match value {
            KeyCode::Key1 => WinitKeyCode::Digit1,
            KeyCode::Key2 => WinitKeyCode::Digit2,
            KeyCode::Key3 => WinitKeyCode::Digit3,
            KeyCode::Key4 => WinitKeyCode::Digit4,
            KeyCode::Key5 => WinitKeyCode::Digit5,
            KeyCode::Key6 => WinitKeyCode::Digit6,
            KeyCode::Key7 => WinitKeyCode::Digit7,
            KeyCode::Key8 => WinitKeyCode::Digit8,
            KeyCode::Key9 => WinitKeyCode::Digit9,
            KeyCode::Key0 => WinitKeyCode::Digit0,
            KeyCode::A => WinitKeyCode::KeyA,
            KeyCode::B => WinitKeyCode::KeyB,
            KeyCode::C => WinitKeyCode::KeyC,
            KeyCode::D => WinitKeyCode::KeyD,
            KeyCode::E => WinitKeyCode::KeyE,
            KeyCode::F => WinitKeyCode::KeyF,
            KeyCode::G => WinitKeyCode::KeyG,
            KeyCode::H => WinitKeyCode::KeyH,
            KeyCode::I => WinitKeyCode::KeyI,
            KeyCode::J => WinitKeyCode::KeyJ,
            KeyCode::K => WinitKeyCode::KeyK,
            KeyCode::L => WinitKeyCode::KeyL,
            KeyCode::M => WinitKeyCode::KeyM,
            KeyCode::N => WinitKeyCode::KeyN,
            KeyCode::O => WinitKeyCode::KeyO,
            KeyCode::P => WinitKeyCode::KeyP,
            KeyCode::Q => WinitKeyCode::KeyQ,
            KeyCode::R => WinitKeyCode::KeyR,
            KeyCode::S => WinitKeyCode::KeyS,
            KeyCode::T => WinitKeyCode::KeyT,
            KeyCode::U => WinitKeyCode::KeyU,
            KeyCode::V => WinitKeyCode::KeyV,
            KeyCode::W => WinitKeyCode::KeyW,
            KeyCode::X => WinitKeyCode::KeyX,
            KeyCode::Y => WinitKeyCode::KeyY,
            KeyCode::Z => WinitKeyCode::KeyZ,
            KeyCode::Escape => WinitKeyCode::Escape,
            KeyCode::F1 => WinitKeyCode::F1,
            KeyCode::F2 => WinitKeyCode::F2,
            KeyCode::F3 => WinitKeyCode::F3,
            KeyCode::F4 => WinitKeyCode::F4,
            KeyCode::F5 => WinitKeyCode::F5,
            KeyCode::F6 => WinitKeyCode::F6,
            KeyCode::F7 => WinitKeyCode::F7,
            KeyCode::F8 => WinitKeyCode::F8,
            KeyCode::F9 => WinitKeyCode::F9,
            KeyCode::F10 => WinitKeyCode::F10,
            KeyCode::F11 => WinitKeyCode::F11,
            KeyCode::F12 => WinitKeyCode::F12,
            KeyCode::F13 => WinitKeyCode::F13,
            KeyCode::F14 => WinitKeyCode::F14,
            KeyCode::F15 => WinitKeyCode::F15,
            KeyCode::F16 => WinitKeyCode::F16,
            KeyCode::F17 => WinitKeyCode::F17,
            KeyCode::F18 => WinitKeyCode::F18,
            KeyCode::F19 => WinitKeyCode::F19,
            KeyCode::F20 => WinitKeyCode::F20,
            KeyCode::F21 => WinitKeyCode::F21,
            KeyCode::F22 => WinitKeyCode::F22,
            KeyCode::F23 => WinitKeyCode::F23,
            KeyCode::F24 => WinitKeyCode::F24,
            KeyCode::PrintScreen => WinitKeyCode::PrintScreen,
            KeyCode::ScrollLock => WinitKeyCode::ScrollLock,
            KeyCode::Pause => WinitKeyCode::Pause,
            KeyCode::Insert => WinitKeyCode::Insert,
            KeyCode::Home => WinitKeyCode::Home,
            KeyCode::Delete => WinitKeyCode::Delete,
            KeyCode::End => WinitKeyCode::End,
            KeyCode::PageDown => WinitKeyCode::PageDown,
            KeyCode::PageUp => WinitKeyCode::PageUp,
            KeyCode::Left => WinitKeyCode::ArrowLeft,
            KeyCode::Up => WinitKeyCode::ArrowUp,
            KeyCode::Right => WinitKeyCode::ArrowRight,
            KeyCode::Down => WinitKeyCode::ArrowDown,
            KeyCode::Backspace => WinitKeyCode::Backspace,
            KeyCode::Enter => WinitKeyCode::Enter,
            KeyCode::Space => WinitKeyCode::Space,
            KeyCode::NumLock => WinitKeyCode::NumLock,
            KeyCode::Numpad0 => WinitKeyCode::Numpad0,
            KeyCode::Numpad1 => WinitKeyCode::Numpad1,
            KeyCode::Numpad2 => WinitKeyCode::Numpad2,
            KeyCode::Numpad3 => WinitKeyCode::Numpad3,
            KeyCode::Numpad4 => WinitKeyCode::Numpad4,
            KeyCode::Numpad5 => WinitKeyCode::Numpad5,
            KeyCode::Numpad6 => WinitKeyCode::Numpad6,
            KeyCode::Numpad7 => WinitKeyCode::Numpad7,
            KeyCode::Numpad8 => WinitKeyCode::Numpad8,
            KeyCode::Numpad9 => WinitKeyCode::Numpad9,
            KeyCode::Backslash => WinitKeyCode::Backslash,
            KeyCode::CapsLock => WinitKeyCode::CapsLock,
            KeyCode::Comma => WinitKeyCode::Comma,
            KeyCode::Convert => WinitKeyCode::Convert,
            KeyCode::Equal => WinitKeyCode::Equal,
            KeyCode::AltLeft => WinitKeyCode::AltLeft,
            KeyCode::BracketLeft => WinitKeyCode::BracketLeft,
            KeyCode::ControlLeft => WinitKeyCode::ControlLeft,
            KeyCode::ShiftLeft => WinitKeyCode::ShiftLeft,
            KeyCode::SuperLeft => WinitKeyCode::SuperLeft,
            KeyCode::LaunchMail => WinitKeyCode::LaunchMail,
            KeyCode::MediaSelect => WinitKeyCode::MediaSelect,
            KeyCode::Minus => WinitKeyCode::Minus,
            KeyCode::AudioVolumeMute => WinitKeyCode::AudioVolumeMute,
            KeyCode::MediaTrackNext => WinitKeyCode::MediaTrackNext,
            KeyCode::NonConvert => WinitKeyCode::NonConvert,
            KeyCode::NumpadComma => WinitKeyCode::NumpadComma,
            KeyCode::NumpadEnter => WinitKeyCode::NumpadEnter,
            KeyCode::NumpadEqual => WinitKeyCode::NumpadEqual,
            KeyCode::Period => WinitKeyCode::Period,
            KeyCode::MediaPlayPause => WinitKeyCode::MediaPlayPause,
            KeyCode::Power => WinitKeyCode::Power,
            KeyCode::MediaTrackPrevious => WinitKeyCode::MediaTrackPrevious,
            KeyCode::AltRight => WinitKeyCode::AltRight,
            KeyCode::BracketRight => WinitKeyCode::BracketRight,
            KeyCode::ControlRight => WinitKeyCode::ControlRight,
            KeyCode::ShiftRight => WinitKeyCode::ShiftRight,
            KeyCode::SuperRight => WinitKeyCode::SuperRight,
            KeyCode::Semicolon => WinitKeyCode::Semicolon,
            KeyCode::Slash => WinitKeyCode::Slash,
            KeyCode::Sleep => WinitKeyCode::Sleep,
            KeyCode::MediaStop => WinitKeyCode::MediaStop,
            KeyCode::Tab => WinitKeyCode::Tab,
            KeyCode::AudioVolumeDown => WinitKeyCode::AudioVolumeDown,
            KeyCode::AudioVolumeUp => WinitKeyCode::AudioVolumeUp,
            KeyCode::WakeUp => WinitKeyCode::WakeUp,
            KeyCode::BrowserBack => WinitKeyCode::BrowserBack,
            KeyCode::BrowserFavorites => WinitKeyCode::BrowserFavorites,
            KeyCode::BrowserForward => WinitKeyCode::BrowserForward,
            KeyCode::BrowserHome => WinitKeyCode::BrowserHome,
            KeyCode::BrowserRefresh => WinitKeyCode::BrowserRefresh,
            KeyCode::BrowserSearch => WinitKeyCode::BrowserSearch,
            KeyCode::BrowserStop => WinitKeyCode::BrowserStop,
            KeyCode::Copy => WinitKeyCode::Copy,
            KeyCode::Paste => WinitKeyCode::Paste,
            KeyCode::Cut => WinitKeyCode::Cut,
            KeyCode::NumpadAdd => WinitKeyCode::NumpadAdd,
            KeyCode::NumpadDivide => WinitKeyCode::NumpadDivide,
            KeyCode::NumpadDecimal => WinitKeyCode::NumpadDecimal,
            KeyCode::NumpadMultiply => WinitKeyCode::NumpadMultiply,
            KeyCode::NumpadSubtract => WinitKeyCode::NumpadSubtract,
            KeyCode::Backquote => WinitKeyCode::Backquote,
            KeyCode::NumpadBackspace => WinitKeyCode::NumpadBackspace,
            KeyCode::NumpadClear => WinitKeyCode::NumpadClear,
            KeyCode::NumpadClearEntry => WinitKeyCode::NumpadClearEntry,
            KeyCode::NumpadHash => WinitKeyCode::NumpadHash,
            KeyCode::NumpadMemoryAdd => WinitKeyCode::NumpadMemoryAdd,
            KeyCode::NumpadMemoryClear => WinitKeyCode::NumpadMemoryClear,
            KeyCode::NumpadMemoryRecall => WinitKeyCode::NumpadMemoryRecall,
            KeyCode::NumpadMemoryStore => WinitKeyCode::NumpadMemoryStore,
            KeyCode::NumpadMemorySubtract => WinitKeyCode::NumpadMemorySubtract,
            KeyCode::NumpadParenLeft => WinitKeyCode::NumpadParenLeft,
            KeyCode::NumpadParenRight => WinitKeyCode::NumpadParenRight,
            KeyCode::NumpadStar => WinitKeyCode::NumpadStar,
            KeyCode::IntlBackslash => WinitKeyCode::IntlBackslash,
            KeyCode::IntlRo => WinitKeyCode::IntlRo,
            KeyCode::IntlYen => WinitKeyCode::IntlYen,
            KeyCode::Quote => WinitKeyCode::Quote,
            KeyCode::ContextMenu => WinitKeyCode::ContextMenu,
            KeyCode::KanaMode => WinitKeyCode::KanaMode,
            KeyCode::Lang1 => WinitKeyCode::Lang1,
            KeyCode::Lang2 => WinitKeyCode::Lang2,
            KeyCode::Lang3 => WinitKeyCode::Lang3,
            KeyCode::Lang4 => WinitKeyCode::Lang4,
            KeyCode::Lang5 => WinitKeyCode::Lang5,
            KeyCode::Help => WinitKeyCode::Help,
            KeyCode::Fn => WinitKeyCode::Fn,
            KeyCode::FnLock => WinitKeyCode::FnLock,
            KeyCode::Eject => WinitKeyCode::Eject,
            KeyCode::LaunchApp1 => WinitKeyCode::LaunchApp1,
            KeyCode::LaunchApp2 => WinitKeyCode::LaunchApp2,
            KeyCode::Meta => WinitKeyCode::Meta,
            KeyCode::Hyper => WinitKeyCode::Hyper,
            KeyCode::Turbo => WinitKeyCode::Turbo,
            KeyCode::Abort => WinitKeyCode::Abort,
            KeyCode::Resume => WinitKeyCode::Resume,
            KeyCode::Suspend => WinitKeyCode::Suspend,
            KeyCode::Again => WinitKeyCode::Again,
            KeyCode::Find => WinitKeyCode::Find,
            KeyCode::Open => WinitKeyCode::Open,
            KeyCode::Props => WinitKeyCode::Props,
            KeyCode::Select => WinitKeyCode::Select,
            KeyCode::Undo => WinitKeyCode::Undo,
            KeyCode::Hiragana => WinitKeyCode::Hiragana,
            KeyCode::Katakana => WinitKeyCode::Katakana,
            KeyCode::F25 => WinitKeyCode::F25,
            KeyCode::F26 => WinitKeyCode::F26,
            KeyCode::F27 => WinitKeyCode::F27,
            KeyCode::F28 => WinitKeyCode::F28,
            KeyCode::F29 => WinitKeyCode::F29,
            KeyCode::F30 => WinitKeyCode::F30,
            KeyCode::F31 => WinitKeyCode::F31,
            KeyCode::F32 => WinitKeyCode::F32,
            KeyCode::F33 => WinitKeyCode::F33,
            KeyCode::F34 => WinitKeyCode::F34,
            KeyCode::F35 => WinitKeyCode::F35,
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
    pub keys: FxHashMap<WinitKeyCode, bool>,
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
        let key_code = WinitKeyCode::from(key_code);
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

    pub fn update_key_states(&mut self, key_code: WinitKeyCode, state: ElementState) {
        log::trace!("update_key_states: {:?} {:?}", key_code, state);
        *self.keys.entry(key_code).or_insert(false) = state == ElementState::Pressed;
    }

    pub fn update_mouse_button_states(&mut self, button: WinitMouseButton, state: ElementState) {
        log::trace!("update_mouse_button_states: {:?} {:?}", button, state);
        *self.btns.entry(button).or_insert(false) = state == ElementState::Pressed;
    }

    pub fn update_modifier_states(&mut self, modifiers: &Modifiers) {
        log::trace!("update_modifier_states: {:?}", modifiers.state());
        self.mods = modifiers.state();
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
        self.is_key_pressed(KeyCode::ShiftLeft) || self.is_key_pressed(KeyCode::ShiftRight)
    }

    pub fn is_left_shift_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::ShiftLeft)
    }

    pub fn is_right_shift_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::ShiftRight)
    }

    pub fn is_ctrl_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::ControlLeft) || self.is_key_pressed(KeyCode::ControlRight)
    }

    pub fn is_left_ctrl_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::ControlLeft)
    }

    pub fn is_right_ctrl_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::ControlRight)
    }

    pub fn is_alt_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::AltLeft) || self.is_key_pressed(KeyCode::AltRight)
    }

    pub fn is_left_alt_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::AltLeft)
    }

    pub fn is_right_alt_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::AltRight)
    }

    pub fn is_super_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::SuperLeft) || self.is_key_pressed(KeyCode::SuperRight)
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
