use std::{collections::HashMap, fs::File, io::Read};

use ab_glyph::{Font as AbFont, FontArc, GlyphId, PxScaleFont, ScaleFont};
use anyhow::Result;

use crate::{
    asset_loader::AssetLoader,
    builder_field,
    ui::primitives::{Rect, Tile},
    vec4, Vec4,
};

mod layout;
mod rasterize;

#[derive(Debug, Clone, Default)]
pub struct FontConfig {
    pub regular: Option<String>,
    pub bold: Option<String>,
    pub light: Option<String>,
    pub medium: Option<String>,
}

impl FontConfig {
    pub fn regular(mut self, path: String) -> Self {
        self.regular = Some(path);
        self
    }

    pub fn bold(mut self, path: String) -> Self {
        self.bold = Some(path);
        self
    }

    pub fn light(mut self, path: String) -> Self {
        self.light = Some(path);
        self
    }

    pub fn medium(mut self, path: String) -> Self {
        self.medium = Some(path);
        self
    }
}

#[derive(Debug, Clone)]
pub struct FontFamily {
    pub regular: Font,
    pub bold: Font,
    pub light: Font,
    pub medium: Font,
}

impl FontFamily {
    pub fn new(
        config: FontConfig,
        scale: f32,
        asset_loader: &mut AssetLoader,
    ) -> Result<Self> {
        Ok(Self {
            regular: Font::from_file(config.regular, FontWeight::Regular, scale, asset_loader)?,
            bold: Font::from_file(config.bold, FontWeight::Bold, scale, asset_loader)?,
            light: Font::from_file(config.light, FontWeight::Light, scale, asset_loader)?,
            medium: Font::from_file(config.medium, FontWeight::Medium, scale, asset_loader)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FontWeight {
    Regular,
    Bold,
    Light,
    Medium,
}

#[derive(Debug, Clone)]
pub struct Font {
    font: PxScaleFont<FontArc>,
    texture_index: i32,
    glyph_texture_coords: HashMap<GlyphId, Rect>,
    text_color: Vec4,
}

impl Font {
    builder_field!(text_color, Vec4);

    fn from_file(
        path: Option<String>,
        weight: FontWeight,
        scale: f32,
        asset_loader: &mut AssetLoader,
    ) -> Result<Self> {
        let bytes = match path {
            Some(path) => {
                let mut buffer = vec![];
                File::open(path)?.read_to_end(&mut buffer)?;
                buffer
            }
            None => match weight {
                FontWeight::Regular => include_bytes!("default/Roobert-Regular.ttf").to_vec(),
                FontWeight::Bold => include_bytes!("default/Roobert-Bold.ttf").to_vec(),
                FontWeight::Light => include_bytes!("default/Roobert-Light.ttf").to_vec(),
                FontWeight::Medium => include_bytes!("default/Roobert-Medium.ttf").to_vec(),
            },
        };
        let font = FontArc::try_from_vec(bytes)?;
        Self::from_ab_glyph_font(font.into_scaled(scale), asset_loader)
    }

    pub fn rescale(
        self,
        scale: f32,
        asset_loader: &mut AssetLoader,
    ) -> Result<Self> {
        let rescaled_font = self.font.with_scale(scale);
        let font = Self::from_ab_glyph_font(rescaled_font, asset_loader)?;
        Ok(Self {
            font: font.font,
            texture_index: font.texture_index,
            glyph_texture_coords: font.glyph_texture_coords,
            ..self
        })
    }

    pub fn from_ab_glyph_font(
        font: PxScaleFont<FontArc>,
        asset_loader: &mut AssetLoader,
    ) -> Result<Self> {
        let glyphs = Self::layout_chars(
            &font,
            10,
            2048,
            font.codepoint_ids().map(|(_id, char)| char),
        );

        let (rasterized_glyphs, glyph_texture_coords) =
            Self::rasterize_glyphs(&font, &glyphs);

        let texture_index =
            asset_loader.create_texture_with_data(&[rasterized_glyphs])?;

        Ok(Self {
            font,
            texture_index,
            glyph_texture_coords,
            text_color: vec4(1.0, 1.0, 1.0, 1.0),
        })
    }

    pub fn build_text_tiles<T>(&self, content: T) -> (Vec<Tile>, Rect)
    where
        T: AsRef<str>,
    {
        let glyphs = Self::layout_text(&self.font, content);
        let mut tiles = Vec::with_capacity(glyphs.len());
        let mut total_bounds: Option<Rect> = None;

        glyphs
            .into_iter()
            .filter_map(|glyph| {
                self.font
                    .outline_glyph(glyph.clone())
                    .map(|outline| (glyph, outline))
            })
            .filter_map(|(glyph, outline)| {
                self.glyph_texture_coords
                    .get(&glyph.id)
                    .map(|tex_coords| (glyph, *tex_coords, outline))
            })
            .for_each(|(glyph, texture_coords, outline)| {
                let bounds = outline.px_bounds();
                let tile = Tile {
                    model: Rect::new(
                        bounds.min.y.round(),
                        bounds.min.x.round(),
                        bounds.max.y.round(),
                        bounds.max.x.round(),
                    ),
                    uv: texture_coords,
                    texture_index: self.texture_index,
                    color: self.text_color,
                    ..Default::default()
                };
                tiles.push(tile);

                let glyph_bounds: Rect = self.font.glyph_bounds(&glyph).into();
                if let Some(total) = total_bounds.take() {
                    total_bounds = Some(total.expand(glyph_bounds));
                } else {
                    total_bounds = Some(glyph_bounds);
                }
            });

        (tiles, total_bounds.unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0)))
    }

    pub fn line_height(&self) -> f32 {
        self.font.height()
    }
}

impl From<ab_glyph::Rect> for Rect {
    fn from(rect: ab_glyph::Rect) -> Self {
        Rect::new(rect.min.y, rect.min.x, rect.max.y, rect.max.x)
    }
}