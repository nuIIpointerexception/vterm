use vui::Vec4;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub [u8; 3]);

impl Color {
    pub const BLACK: Self = Self::from_rgb(46, 51, 63);
    pub const DARK_GRAY: Self = Self::from_rgb(96, 96, 96);
    pub const GRAY: Self = Self::from_rgb(160, 160, 160);
    pub const LIGHT_GRAY: Self = Self::from_rgb(220, 220, 220);
    pub const WHITE: Self = Self::from_rgb(229, 232, 239);
    pub const MAGENTA: Self = Self::from_rgb(180, 141, 172);
    pub const CYAN: Self = Self::from_rgb(136, 191, 207);

    pub const BROWN: Self = Self::from_rgb(165, 42, 42);
    pub const DARK_RED: Self = Self::from_rgb(0x8B, 0, 0);
    pub const RED: Self = Self::from_rgb(190, 96, 105);
    pub const LIGHT_RED: Self = Self::from_rgb(255, 128, 128);

    pub const YELLOW: Self = Self::from_rgb(235, 202, 138);
    pub const LIGHT_YELLOW: Self = Self::from_rgb(170, 170, 0xE0);
    pub const KHAKI: Self = Self::from_rgb(240, 230, 140);

    pub const DARK_GREEN: Self = Self::from_rgb(0, 0x64, 0);
    pub const GREEN: Self = Self::from_rgb(163, 189, 139);
    pub const LIGHT_GREEN: Self = Self::from_rgb(0x90, 0xEE, 0x90);

    pub const DARK_BLUE: Self = Self::from_rgb(0, 0, 0x8B);
    pub const BLUE: Self = Self::from_rgb(129, 160, 192);
    pub const LIGHT_BLUE: Self = Self::from_rgb(0xAD, 0xD8, 0xE6);

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b])
    }
}

impl From<Color> for Vec4 {
    fn from(c: Color) -> Self {
        Vec4::new(
            c.0[0] as f32 / 255.0,
            c.0[1] as f32 / 255.0,
            c.0[2] as f32 / 255.0,
            1.0,
        )
    }
}
