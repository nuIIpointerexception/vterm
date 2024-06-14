use vui::{ui::font::FontConfig, vec4};
use ::vui::{
    asset_loader::AssetLoader,
    ui::{UIState, font::FontFamily, widgets::prelude::*},
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TerminalMessage {
    // Add messages here.
}

pub struct Terminal {
    em: f32,
    font: FontFamily,
}

impl Terminal {
    pub fn new(content_scale: f32, asset_loader: Option<&mut AssetLoader>) -> anyhow::Result<Self> {
        let em = 16.0 * content_scale;
        let asset_loader = asset_loader.unwrap();
        
        // We don't have a config system at the moment, so we intitialize the default fonts.
        let font = FontFamily::new(FontConfig::default(), 2.0 * em, asset_loader).unwrap();

        Ok(Self { em, font})
    }
}

impl UIState for Terminal {
    type Message = TerminalMessage;

    fn view(&self) -> Element<Self::Message> {
        align(
            col()
                .child(
                    label(
                        &self.font.bold,
                        "bold",
                    ).color(vec4(1.0, 0.0, 0.0, 1.0)),
                    Justify::Center,
                )
                .child(
                    label(
                        &self.font.medium,
                        "medium",
                    ),
                    Justify::Center,
                )
                .child(
                    label(
                        &self.font.regular,
                        "regular",
                    ),
                    Justify::Center,
                )
                .child(
                    label(
                        &self.font.light,
                        "light",
                    ),
                    Justify::Center,
                )
                .space_between(SpaceBetween::Fixed(self.em / 4.0))
                .container()
                .padding(0.5 * self.em)
                .max_width(Constraint::FixedMaxSize(10.0 * self.em)),
        )
        .alignment(HAlignment::Center, VAlignment::Center)
        .into()
    }

    fn update(&mut self, message: &TerminalMessage) {
        match *message {
            // handle stuff here.
        }
    }
}