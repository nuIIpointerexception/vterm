use std::sync::mpsc::Sender;

use winit::{
    event::KeyEvent,
    keyboard::{Key, ModifiersState, NamedKey},
};

/// InputState processes input events and sends them to the terminal.
pub struct InputState {
    rtx: Sender<Vec<u8>>,
}

impl InputState {
    pub fn new(rtx: Sender<Vec<u8>>) -> Self {
        Self { rtx }
    }

    pub fn apply_keyboard(&mut self, input: KeyEvent, mods: &ModifiersState) {
        let text = match input.logical_key {
            Key::Named(NamedKey::Backspace) => Some("\u{8}"),
            Key::Named(NamedKey::Enter) => Some("\r"),
            Key::Named(NamedKey::ArrowUp) => Some("\x1bOA"),
            Key::Named(NamedKey::ArrowDown) => Some("\x1bOB"),
            Key::Named(NamedKey::ArrowRight) => Some("\x1bOC"),
            Key::Named(NamedKey::ArrowLeft) => Some("\x1bOD"),
            Key::Named(NamedKey::Tab) => Some("\t"),
            Key::Named(NamedKey::Escape) => Some("\x1b"),
            _ => {
                if mods.control_key() {
                    let n = input
                        .logical_key
                        .to_text()
                        .unwrap()
                        .chars()
                        .next()
                        .unwrap();
                    let mut m = n as u8;
                    m &= 0b1001_1111;
                    let _ = self.rtx.send(vec![m]);
                    None
                } else {
                    Some(input.logical_key.to_text().unwrap())
                }
            }
        };
        if let Some(text) = text {
            let _ = self.rtx.send(text.into());
        };
    }
}
