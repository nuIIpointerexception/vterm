use ::vui::{
    asset_loader::AssetLoader,
    ui::{font::FontFamily, widgets::prelude::*, UIState},
};
use vui::ui::{color::Color, font::FontConfig};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TerminalMessage {
    // Add messages here.
}

pub struct Terminal {
    em: f32,
    label_1: Element<TerminalMessage>,
    label_2: Element<TerminalMessage>,
    image: Element<TerminalMessage>,
    label_3: Element<TerminalMessage>,
}

impl Terminal {
    pub fn new(content_scale: f32, asset_loader: &mut AssetLoader) -> anyhow::Result<Self> {
        let em = 16.0 * content_scale;
        let font_32 = FontFamily::new(FontConfig::default(), 2.0 * em, asset_loader)?;
        let font_64 = FontFamily::new(FontConfig::default(), 4.0 * em, asset_loader)?;

        let image_dir = dirs::home_dir().unwrap().join(".vterm/assets/images/");
        let img_rust = asset_loader.read_texture(image_dir.join("rust.png"))?;
        let label_1 = label(
            &font_64.medium,
            "
            Powered by an easy-to-use,
            developer friendly gui library",
        )
        .gradient(Color::new(1.0, 1.0, 1.0, 1.0), Color::new(1.0, 1.0, 1.0, 0.3))
        .into();

        let label_2 = label(
            &font_32.light,
            "
            vterm was built to save developer time
            and allow highly customizable terminal workflows.
            Use extensions and themes to make your terminal
            look and feel like you want it to.",
        )
        .gradient(Color::new(1.0, 1.0, 1.0, 1.0), Color::new(1.0, 1.0, 1.0, 0.3))
        .into();

        let image = img(em * 20.0, em * 20.0, img_rust).into();

        let label_3 = label(&font_32.light, "Multi-window/tab ai and gpu accelerated.")
            .gradient(Color::new(1.0, 1.0, 1.0, 1.0), Color::new(1.0, 1.0, 1.0, 0.3))
            .into();

        Ok(Self { em, label_1, label_2, image, label_3 })
    }
}

impl UIState for Terminal {
    type Message = TerminalMessage;

    fn view(&mut self) -> Element<Self::Message> {
        align(
            col()
                .child(self.label_1.clone())
                .child(self.label_2.clone())
                .child(self.image.clone())
                .child(self.label_3.clone())
                .space_between(SpaceBetween::Fixed(self.em))
                .container()
                .padding(0.5 * self.em),
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
