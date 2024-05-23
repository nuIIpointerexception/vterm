use ::vui::{
    asset_loader::AssetLoader,
    ui::{UIState, widgets::prelude::*},
};

use crate::constants::{get_fps, get_gpu};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TerminalMessage {
    ToggleFullscreen,
}

pub struct Terminal {
    em: f32,
    font: Font,
    is_fullscreen: bool,
}

impl Terminal {
    pub fn new(
        content_scale: f32,
        asset_loader: &mut AssetLoader,
    ) -> anyhow::Result<Self> {
        let em = 16.0 * content_scale;
        let font = Font::from_font_file(
            "/usr/share/fonts/TTF/zed-sans-extended.ttf",
            1.0 * em,
            asset_loader,
        )?;
        Ok(Self {
            em,
            font,
            is_fullscreen: false,
        })
    }
}

impl UIState for Terminal {
    type Message = TerminalMessage;

    fn view(&self) -> Element<Self::Message> {
        align(
            col()
                .child(label(&self.font, "vterm"), Justify::Begin)
                .child(
                    label(&self.font, format!("{}", get_gpu().as_ref())),
                    Justify::Begin,
                )
                .child(
                    label(&self.font, format!("fps: {:.2}", get_fps())),
                    Justify::Begin,
                )
                .space_between(SpaceBetween::Fixed(self.em / 4.0))
                .container()
                .padding(0.5 * self.em)
                .max_width(Constraint::FixedMaxSize(10.0 * self.em)),
        )
        .alignment(HAlignment::Left, VAlignment::Top)
        .into()
    }

    fn update(&mut self, message: &TerminalMessage) {
        match *message {
            TerminalMessage::ToggleFullscreen => {
                self.is_fullscreen = !self.is_fullscreen;
            }
        }
    }
}
