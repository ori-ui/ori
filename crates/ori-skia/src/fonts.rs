use std::{hash::BuildHasherDefault, num::NonZeroUsize};

use lru::LruCache;
use ori_core::{
    layout::{Point, Rect, Size},
    text::{
        FontFamily, FontSource, FontStretch, FontStyle, Fonts, GlyphCluster, Paragraph, TextAlign,
        TextDirection, TextLayoutLine, TextWrap,
    },
};
use seahash::SeaHasher;
use skia_safe::{
    font_style::{FontStyle as SkiaFontStyle, Slant, Weight, Width},
    textlayout::{
        FontCollection, Paragraph as SkiaParagraph, ParagraphBuilder, ParagraphStyle,
        TextAlign as SkiaTextAlign, TextDirection as SkiaTextDirection, TextStyle,
        TypefaceFontProvider,
    },
    FontMgr,
};

use crate::SkiaRenderer;

#[allow(dead_code)]
pub struct SkiaFonts {
    collection: FontCollection,
    provider: TypefaceFontProvider,
    manager: FontMgr,
    paragraph_cache: LruCache<Paragraph, SkiaParagraph, BuildHasherDefault<SeaHasher>>,
}

impl SkiaFonts {
    pub fn new(default_font: Option<&str>) -> Self {
        let mut collection = FontCollection::new();
        let provider = TypefaceFontProvider::new();
        let manager = FontMgr::new();

        collection.set_dynamic_font_manager(FontMgr::clone(&provider));
        collection.set_default_font_manager(manager.clone(), default_font);

        let cache_size = NonZeroUsize::new(128).unwrap();
        let paragraph_cache = LruCache::with_hasher(cache_size, Default::default());

        Self {
            collection,
            provider,
            manager,
            paragraph_cache,
        }
    }

    pub fn build_skia_paragraph(&mut self, paragraph: &Paragraph) -> &mut SkiaParagraph {
        if self.paragraph_cache.contains(paragraph) {
            return self.paragraph_cache.get_mut(paragraph).unwrap();
        }

        let mut style = ParagraphStyle::new();

        let align = match paragraph.align {
            TextAlign::Start => SkiaTextAlign::Left,
            TextAlign::Center => SkiaTextAlign::Center,
            TextAlign::End => SkiaTextAlign::Right,
        };

        style.set_height(paragraph.line_height);
        style.set_text_align(align);

        if let TextWrap::None = paragraph.wrap {
            style.set_max_lines(1);
        }

        let mut builder = ParagraphBuilder::new(&style, &self.collection);

        for (text, attributes) in paragraph.iter() {
            let mut style = TextStyle::new();

            let family = match &attributes.family {
                FontFamily::Name(name) => name.as_str(),
                FontFamily::Serif => "Roboto",
                FontFamily::SansSerif => "Roboto",
                FontFamily::Monospace => "Roboto Mono",
                FontFamily::Cursive => "Roboto",
                FontFamily::Fantasy => "Roboto",
            };

            let weight = Weight::from(attributes.weight.0 as i32);

            let width = match attributes.stretch {
                FontStretch::UltraCondensed => Width::ULTRA_CONDENSED,
                FontStretch::ExtraCondensed => Width::EXTRA_CONDENSED,
                FontStretch::Condensed => Width::CONDENSED,
                FontStretch::SemiCondensed => Width::SEMI_CONDENSED,
                FontStretch::Normal => Width::NORMAL,
                FontStretch::SemiExpanded => Width::SEMI_EXPANDED,
                FontStretch::Expanded => Width::EXPANDED,
                FontStretch::ExtraExpanded => Width::EXTRA_EXPANDED,
                FontStretch::UltraExpanded => Width::ULTRA_EXPANDED,
            };

            let slant = match attributes.style {
                FontStyle::Normal => Slant::Upright,
                FontStyle::Italic => Slant::Italic,
                FontStyle::Oblique => Slant::Oblique,
            };

            let font_style = SkiaFontStyle::new(weight, width, slant);

            style.set_font_size(attributes.size);
            style.set_font_families(&[family]);
            style.set_font_style(font_style);
            style.set_color(SkiaRenderer::skia_color(attributes.color));

            if !attributes.ligatures {
                // disable ligatures
                style.add_font_feature("liga", 0);
                style.add_font_feature("clig", 0);
            }

            builder.push_style(&style);
            builder.add_text(text);
            builder.pop();
        }

        self.paragraph_cache.put(paragraph.clone(), builder.build());
        self.paragraph_cache.get_mut(paragraph).unwrap()
    }
}

impl Fonts for SkiaFonts {
    fn load(&mut self, source: FontSource<'_>, name: Option<&str>) {
        let fonts = source.data().unwrap();

        for data in fonts {
            if let Some(typeface) = self.manager.new_from_data(&data, None) {
                self.provider.register_typeface(typeface, name);
            }
        }
    }

    fn layout(&mut self, paragraph: &Paragraph, width: f32) -> Vec<TextLayoutLine> {
        let skia_paragraph = self.build_skia_paragraph(paragraph);
        skia_paragraph.layout(width);

        let mut lines = Vec::new();

        let metrics = skia_paragraph.get_line_metrics();

        for (i, metric) in metrics.iter().enumerate() {
            // the following code is a revulting mess of special cases to handle
            // i do not ever want to have to know that every line except the last
            // has contains the newline character that caused it to wrap.
            //
            // this sucks and i hate it. thanks google. you wasted several hours of my
            // life with this garbage.
            //
            //  - Hjalte, 2024-12-03

            let end = metric.end_including_newline.saturating_sub(1);

            let has_newline = if paragraph.text().is_char_boundary(end) {
                paragraph.text()[end..].starts_with('\n')
            } else {
                false
            };

            let is_last = i == metrics.len() - 1;

            let range = if has_newline {
                if is_last {
                    metric.start_index + 1..metric.end_including_newline
                } else {
                    metric.start_index..end
                }
            } else {
                metric.start_index..metric.end_including_newline
            };

            let mut line = TextLayoutLine {
                left: metric.left as f32,
                ascent: metric.ascent as f32,
                descent: metric.descent as f32,
                width: metric.width as f32,
                height: metric.height as f32,
                baseline: metric.baseline as f32,
                range: range.clone(),
                glyphs: Vec::new(),
            };

            for i in metric.start_index..metric.end_index {
                let Some(glyph) = skia_paragraph.get_glyph_cluster_at(i) else {
                    continue;
                };

                if &paragraph.text()[glyph.text_range.clone()] == "\n" {
                    // we don't want to include newline characters in the glyph list
                    // they just confuse everything.

                    continue;
                }

                let bounds = Rect {
                    min: Point::new(glyph.bounds.left, glyph.bounds.top),
                    max: Point::new(glyph.bounds.right, glyph.bounds.bottom),
                };

                let direction = match glyph.position {
                    SkiaTextDirection::LTR => TextDirection::Ltr,
                    SkiaTextDirection::RTL => TextDirection::Rtl,
                };

                line.glyphs.push(GlyphCluster {
                    bounds,
                    range: glyph.text_range,
                    direction,
                });
            }

            lines.push(line);
        }

        lines
    }

    fn measure(&mut self, paragraph: &Paragraph, width: f32) -> Size {
        let skia_paragraph = self.build_skia_paragraph(paragraph);
        skia_paragraph.layout(width);

        let width = skia_paragraph.max_intrinsic_width();
        let height = skia_paragraph.height();

        Size::new(width, height)
    }
}
