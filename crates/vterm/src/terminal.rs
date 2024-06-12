use ::vui::{
    asset_loader::AssetLoader,
    ui::{UIState, widgets::prelude::*},
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TerminalMessage {
    // Add messages here.
}

pub struct Terminal {
    em: f32,
    font: Font,
}

impl Terminal {
    pub fn new(
        content_scale: f32,
        asset_loader: Option<&mut AssetLoader>,
    ) -> anyhow::Result<Self> {
        let em = 16.0 * content_scale;
        let font = Font::from_font_file(
            "/usr/share/fonts/TTF/zed-sans-extended.ttf",
            2.0 * em,
            asset_loader.unwrap(),
        )?;
        Ok(Self { em, font })
    }
}

impl UIState for Terminal {
    type Message = TerminalMessage;

    fn view(&self) -> Element<Self::Message> {
        align(
            col()
                .child(label(&self.font, "vterm"), Justify::Begin)
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
            // handle stuff here.
        }
    }
}
