use std::collections::HashMap;

use ab_glyph::{FontArc, Glyph, GlyphId, PxScaleFont, ScaleFont};

use crate::{
    asset_loader::MipmapData,
    ui::{primitives::Rect, Font},
};

impl Font {
    pub(super) fn layout_chars<T>(
        font: &PxScaleFont<FontArc>,
        padding: u32,
        max_width: u32,
        content: T,
    ) -> Vec<Glyph>
    where
        T: Iterator<Item = char>,
    {
        let v_advance = ((font.line_gap() + font.height()).ceil() as u32) + padding;
        let mut glyphs: Vec<Glyph> = Vec::with_capacity(content.size_hint().0);
        let mut line_number = 1;
        let mut cursor_x = 0.0;
        let mut cursor_y = v_advance as f32;

        for char in content {
            let h_advance = font.h_advance(font.glyph_id(char));

            if ((cursor_x + h_advance) as u32) + padding > max_width {
                line_number += 1;
                cursor_x = 0.0;
                cursor_y = (line_number * v_advance) as f32;
            }

            glyphs.push(
                font.glyph_id(char)
                    .with_scale_and_position(font.scale(), ab_glyph::point(cursor_x, cursor_y)),
            );

            cursor_x += h_advance + (padding as f32);
            cursor_x = cursor_x.round();
            cursor_y = cursor_y.round();
        }

        glyphs
    }

    pub(super) fn rasterize_glyphs(
        font: &PxScaleFont<FontArc>,
        glyphs: &[Glyph],
    ) -> (MipmapData, HashMap<GlyphId, Rect>) {
        let mut max_width = 0;
        let mut max_height = 0;
        let mut outlines = HashMap::with_capacity(glyphs.len());
        for glyph in glyphs {
            if let Some(outline) = font.outline_glyph(glyph.clone()) {
                let bounds = outline.px_bounds();
                max_width = max_width.max(bounds.max.x as u32);
                max_height = max_height.max(bounds.max.y as u32);
                outlines.insert(glyph.id, outline);
            }
        }

        let mut glyph_texture_coords = HashMap::with_capacity(outlines.len());
        let mut rasterized_glyphs =
            MipmapData::allocate(max_width, max_height, [0xff, 0xff, 0xff, 0x00]);

        for (glyph_id, outline) in outlines {
            let bounds = outline.px_bounds();
            let texture_coords = Rect::new(
                bounds.min.y / (max_height as f32),
                bounds.min.x / (max_width as f32),
                bounds.max.y / (max_height as f32),
                bounds.max.x / (max_width as f32),
            );
            let basex = bounds.min.x as u32;
            let basey = bounds.min.y as u32;
            outline.draw(|x, y, coverage| {
                rasterized_glyphs.write_pixel(
                    basex + x,
                    basey + y,
                    [0xff, 0xff, 0xff, (coverage * 255.0) as u8],
                );
            });
            glyph_texture_coords.insert(glyph_id, texture_coords);
        }

        (rasterized_glyphs, glyph_texture_coords)
    }
}
