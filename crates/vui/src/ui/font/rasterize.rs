use ::{
    ab_glyph::{FontArc, Glyph, GlyphId, PxScaleFont, ScaleFont},
    std::collections::HashMap,
};

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
            T: Iterator<Item=char>,
    {
        let v_advance =
            (font.line_gap() + font.height()).ceil() as u32 + padding;

        let mut glyphs = vec![];
        let mut line_number = 1u32;
        let mut cursor = ab_glyph::point(0.0, (line_number * v_advance) as f32);

        for char in content {
            let mut glyph = font.scaled_glyph(char);
            let h_advance = font.h_advance(glyph.id);

            if (cursor.x + h_advance) as u32 + padding > max_width {
                line_number += 1;
                cursor.x = 0.0;
                cursor.y = (line_number * v_advance) as f32;
            }

            glyph.position = cursor;
            glyphs.push(glyph);

            cursor.x += h_advance + padding as f32;
            cursor.x = cursor.x.round();
            cursor.y = cursor.y.round();
        }

        glyphs
    }

    pub(super) fn rasterize_glyphs(
        font: &PxScaleFont<FontArc>,
        glyphs: &[Glyph],
    ) -> (MipmapData, HashMap<GlyphId, Rect>) {
        let mut max_width = 0u32;
        let mut max_height = 0u32;
        let mut outlines = HashMap::with_capacity(glyphs.len());
        for glyph in glyphs {
            let id = glyph.id;
            if let Some(outline) = font.outline_glyph(glyph.clone()) {
                max_width = max_width.max(outline.px_bounds().max.x as u32);
                max_height = max_height.max(outline.px_bounds().max.y as u32);
                outlines.insert(id, outline);
            }
        }

        let mut glyph_texture_coords = HashMap::with_capacity(outlines.len());
        let mut rasterized_glyphs = MipmapData::allocate(
            max_width,
            max_height,
            [0xFF, 0xFF, 0xFF, 0x00],
        );

        for (glyph_id, outline) in outlines {
            let bounds = outline.px_bounds();
            let texture_coords = Rect::new(
                bounds.min.y.round() / max_height as f32,
                bounds.min.x.round() / max_width as f32,
                bounds.max.y.round() / max_height as f32,
                bounds.max.x.round() / max_width as f32,
            );
            let basex = bounds.min.x.round() as u32;
            let basey = bounds.min.y.round() as u32;
            outline.draw(|x, y, coverage| {
                rasterized_glyphs.write_pixel(
                    basex + x,
                    basey + y,
                    [0xFF, 0xFF, 0xFF, (0xFF as f32 * coverage) as u8],
                );
            });
            glyph_texture_coords.insert(glyph_id, texture_coords);
        }

        (rasterized_glyphs, glyph_texture_coords)
    }
}
