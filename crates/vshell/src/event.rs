use std::{
    borrow::Cow,
    fmt::{self, Debug, Formatter},
    sync::Arc,
};

use crate::{term::ClipboardType, vte::ansi::Rgb};

#[derive(Clone)]
pub enum Event {
    MouseCursorDirty,

    Title(String),

    ResetTitle,

    ClipboardStore(ClipboardType, String),

    ClipboardLoad(
        ClipboardType,
        Arc<dyn Fn(&str) -> String + Sync + Send + 'static>,
    ),

    ColorRequest(usize, Arc<dyn Fn(Rgb) -> String + Sync + Send + 'static>),

    PtyWrite(String),

    TextAreaSizeRequest(
        Arc<dyn Fn(WindowSize) -> String + Sync + Send + 'static>,
    ),

    CursorBlinkingChange,

    Wakeup,

    Bell,

    Exit,

    ChildExit(i32),
}

impl Debug for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Event::ClipboardStore(ty, text) => {
                write!(f, "ClipboardStore({ty:?}, {text})")
            }
            Event::ClipboardLoad(ty, _) => write!(f, "ClipboardLoad({ty:?})"),
            Event::TextAreaSizeRequest(_) => write!(f, "TextAreaSizeRequest"),
            Event::ColorRequest(index, _) => write!(f, "ColorRequest({index})"),
            Event::PtyWrite(text) => write!(f, "PtyWrite({text})"),
            Event::Title(title) => write!(f, "Title({title})"),
            Event::CursorBlinkingChange => write!(f, "CursorBlinkingChange"),
            Event::MouseCursorDirty => write!(f, "MouseCursorDirty"),
            Event::ResetTitle => write!(f, "ResetTitle"),
            Event::Wakeup => write!(f, "Wakeup"),
            Event::Bell => write!(f, "Bell"),
            Event::Exit => write!(f, "Exit"),
            Event::ChildExit(code) => write!(f, "ChildExit({code})"),
        }
    }
}

pub trait Notify {
    fn notify<B: Into<Cow<'static, [u8]>>>(&self, _: B);
}

#[derive(Copy, Clone, Debug)]
pub struct WindowSize {
    pub num_lines: u16,
    pub num_cols: u16,
    pub cell_width: u16,
    pub cell_height: u16,
}

pub trait OnResize {
    fn on_resize(&mut self, window_size: WindowSize);
}

pub trait EventListener {
    fn send_event(&self, _event: Event) {}
}

pub struct VoidListener;

impl EventListener for VoidListener {}
