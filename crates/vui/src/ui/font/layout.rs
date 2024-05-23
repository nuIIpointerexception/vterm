use ::ab_glyph::{FontArc, Glyph, PxScaleFont, ScaleFont};

use crate::ui::Font;

impl Font {
    pub(super) fn layout_text<T>(
        font: &PxScaleFont<FontArc>,
        content: T,
    ) -> Vec<Glyph>
        where
            T: AsRef<str>,
    {
        let v_advance = (font.line_gap() + font.height()).ceil() as u32;

        let mut glyphs = vec![];
        let mut line_number = 1u32;
        let mut cursor = ab_glyph::point(0.0, (line_number * v_advance) as f32);

        let mut previous_glyph: Option<Glyph> = None;
        for char in content.as_ref().chars() {
            if char.is_control() {
                if char == '\n' {
                    line_number += 1;
                    cursor.x = 0.0;
                    cursor.y = (line_number * v_advance) as f32;
                }
                previous_glyph = None;
                continue;
            }

            let mut glyph = font.scaled_glyph(char);
            let glyph_id = glyph.id;

            if let Some(previous) = previous_glyph.take() {
                let kern = font.kern(previous.id, glyph.id);
                cursor.x += kern;
            }

            cursor.x = cursor.x.round();
            cursor.y = cursor.y.round();
            glyph.position = cursor;
            glyphs.push(glyph.clone());

            cursor.x += font.h_advance(glyph_id);
        }

        glyphs
    }
}
