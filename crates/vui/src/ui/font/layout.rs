use ::ab_glyph::{FontArc, Glyph, PxScaleFont, ScaleFont};

use crate::ui::Font;

impl Font {
    pub(super) fn layout_text<T: AsRef<str>>(
        font: &PxScaleFont<FontArc>,
        content: T,
    ) -> Vec<Glyph> {
        let v_advance = (font.line_gap() + font.height()).ceil() as u32;
        let mut glyphs = Vec::with_capacity(content.as_ref().len());
        let mut line_number = 1;
        let mut cursor_x = 0.0;
        let mut cursor_y = v_advance as f32;
        let mut prev_glyph_id = None;

        for char in content.as_ref().chars() {
            if char == '\n' {
                line_number += 1;
                cursor_x = 0.0;
                cursor_y = (line_number * v_advance) as f32;
                prev_glyph_id = None;
            } else if !char.is_control() {
                let glyph_id = font.glyph_id(char);
                if let Some(prev_id) = prev_glyph_id {
                    cursor_x += font.kern(prev_id, glyph_id);
                }
                let position =
                    ab_glyph::point(cursor_x.round(), cursor_y.round());
                glyphs.push(
                    font.glyph_id(char)
                        .with_scale_and_position(font.scale(), position),
                );
                cursor_x += font.h_advance(glyph_id);
                prev_glyph_id = Some(glyph_id);
            }
        }

        glyphs
    }
}
