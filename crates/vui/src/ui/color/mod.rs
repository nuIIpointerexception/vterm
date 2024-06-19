mod gradient;
mod style;

pub use gradient::Gradient;
pub use style::Style;

/// A color with red, green, blue and alpha components.
/// The components are in the range [0, 1].
#[derive(Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color with the given red, green, blue and alpha components.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    /// Linearly interpolate between two colors.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let inv_t = 1.0 - t;
        Color::new(
            self.r * inv_t + other.r * t,
            self.g * inv_t + other.g * t,
            self.b * inv_t + other.b * t,
            self.a * inv_t + other.a * t,
        )
    }

    /// Blend two colors together.
    pub fn blend(&self, other: Color) -> Self {
        match other.a {
            a if a >= 1.0 => other,
            a if a <= 0.0 => *self,
            a => Color {
                r: self.r * (1.0 - a) + other.r * a,
                g: self.g * (1.0 - a) + other.g * a,
                b: self.b * (1.0 - a) + other.b * a,
                a: self.a,
            },
        }
    }
}

/// Convert a tuple of four floats to a Color struct.
impl From<(f32, f32, f32, f32)> for Color {
    fn from(tuple: (f32, f32, f32, f32)) -> Self {
        Color::new(tuple.0, tuple.1, tuple.2, tuple.3)
    }
}

/// Convert a hexadecimal color value to a Color struct.
impl From<u32> for Color {
    fn from(hex: u32) -> Self {
        let r = ((hex >> 24) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let b = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let a = (hex & 0xFF) as f32 / 255.0;
        Color { r, g, b, a }
    }
}

/// Convert a Color struct to a hexadecimal color value.
impl From<Color> for u32 {
    fn from(color: Color) -> Self {
        let r = (color.r * 255.0) as u32;
        let g = (color.g * 255.0) as u32;
        let b = (color.b * 255.0) as u32;
        let a = (color.a * 255.0) as u32;
        (r << 24) | (g << 16) | (b << 8) | a
    }
}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "rgba({:#010x})", u32::from(*self))
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn test_color_new() {
        let color = Color::new(0.1, 0.2, 0.3, 0.4);
        assert_eq!(color.r, 0.1);
        assert_eq!(color.g, 0.2);
        assert_eq!(color.b, 0.3);
        assert_eq!(color.a, 0.4);
    }

    #[test]
    fn test_color_lerp() {
        let color1 = Color::new(0.0, 0.0, 0.0, 0.0);
        let color2 = Color::new(1.0, 1.0, 1.0, 1.0);
        let result = color1.lerp(color2, 0.5);
        assert_eq!(result, Color::new(0.5, 0.5, 0.5, 0.5));
    }

    #[test]
    fn test_color_blend() {
        let color1 = Color::new(0.0, 0.0, 0.0, 1.0);
        let color2 = Color::new(1.0, 1.0, 1.0, 0.5);
        let result = color1.blend(color2);
        assert_eq!(result, Color::new(0.5, 0.5, 0.5, 1.0));
    }

    #[test]
    fn test_color_from_tuple() {
        let color: Color = (0.1, 0.2, 0.3, 0.4).into();
        assert_eq!(color, Color::new(0.1, 0.2, 0.3, 0.4));
    }

    #[test]
    fn test_color_from_u32() {
        let color: Color = 0xFF00FF00.into();
        assert_eq!(color, Color::new(1.0, 0.0, 1.0, 0.0));
    }

    #[test]
    fn test_color_into_u32() {
        let color = Color::new(1.0, 0.0, 1.0, 0.0);
        let hex: u32 = color.into();
        assert_eq!(hex, 0xFF00FF00);
    }

    #[test]
    fn test_color_debug() {
        let color = Color::new(1.0, 0.0, 1.0, 0.0);
        assert_eq!(format!("{:?}", color), "rgba(0xff00ff00)");
    }
}
