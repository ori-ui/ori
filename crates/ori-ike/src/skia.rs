use ike::{
    Affine, BorderWidth, Canvas, CornerRadius, FontStretch, FontStyle, Fonts, GlyphCluster, Offset,
    Paint, Paragraph, Point, Rect, Shader, Size, TextDirection, TextLayoutLine, TextStyle,
    TextWrap,
};

pub(crate) struct SkiaFonts {
    pub(crate) collection: skia_safe::textlayout::FontCollection,
}

impl Default for SkiaFonts {
    fn default() -> Self {
        Self::new()
    }
}

impl SkiaFonts {
    pub(crate) fn new() -> Self {
        let mut collection = skia_safe::textlayout::FontCollection::new();
        collection.set_dynamic_font_manager(Some(skia_safe::FontMgr::new()));

        Self { collection }
    }

    fn build_font_style(style: &TextStyle) -> skia_safe::FontStyle {
        let weight = skia_safe::font_style::Weight::from(style.font_weight.0 as i32);

        let width = match style.font_stretch {
            FontStretch::UltraCondensed => skia_safe::font_style::Width::ULTRA_CONDENSED,
            FontStretch::ExtraCondensed => skia_safe::font_style::Width::EXTRA_CONDENSED,
            FontStretch::Condensed => skia_safe::font_style::Width::CONDENSED,
            FontStretch::SemiCondensed => skia_safe::font_style::Width::SEMI_CONDENSED,
            FontStretch::Normal => skia_safe::font_style::Width::NORMAL,
            FontStretch::SemiExpanded => skia_safe::font_style::Width::SEMI_EXPANDED,
            FontStretch::Expanded => skia_safe::font_style::Width::EXPANDED,
            FontStretch::ExtraExpanded => skia_safe::font_style::Width::EXTRA_EXPANDED,
            FontStretch::UltraExpanded => skia_safe::font_style::Width::ULTRA_EXPANDED,
        };

        let slant = match style.font_style {
            FontStyle::Normal => skia_safe::font_style::Slant::Upright,
            FontStyle::Italic => skia_safe::font_style::Slant::Italic,
            FontStyle::Oblique => skia_safe::font_style::Slant::Oblique,
        };
        skia_safe::FontStyle::new(weight, width, slant)
    }

    pub(crate) fn build_paragraph(
        &mut self,
        paragraph: &Paragraph,
    ) -> skia_safe::textlayout::Paragraph {
        let mut style = skia_safe::textlayout::ParagraphStyle::new();

        let align = match paragraph.align {
            ike::TextAlign::Start => skia_safe::textlayout::TextAlign::Start,
            ike::TextAlign::Center => skia_safe::textlayout::TextAlign::Center,
            ike::TextAlign::End => skia_safe::textlayout::TextAlign::End,
        };

        style.set_height(paragraph.line_height);
        style.set_text_align(align);

        if let TextWrap::None = paragraph.wrap {
            style.set_max_lines(1);
        }

        let mut builder = skia_safe::textlayout::ParagraphBuilder::new(&style, &self.collection);

        for (text, style) in paragraph.sections() {
            let mut skia_style = skia_safe::textlayout::TextStyle::new();

            skia_style.set_font_size(style.font_size);
            skia_style.set_font_families(&[&style.font_family]);
            skia_style.set_font_style(Self::build_font_style(style));
            skia_style.set_color(skia_safe::Color::from_argb(
                f32::round(style.color.a * 255.0) as u8,
                f32::round(style.color.r * 255.0) as u8,
                f32::round(style.color.g * 255.0) as u8,
                f32::round(style.color.b * 255.0) as u8,
            ));

            builder.push_style(&skia_style);
            builder.add_text(text);
            builder.pop();
        }

        builder.build()
    }
}

pub(crate) struct SkiaCanvas<'a> {
    pub(crate) canvas: &'a skia_safe::Canvas,
    pub(crate) fonts:  &'a mut SkiaFonts,
}

impl<'a> SkiaCanvas<'a> {
    fn create_paint(&self, paint: &Paint) -> skia_safe::Paint {
        let mut skia_paint = skia_safe::Paint::default();
        skia_paint.set_anti_alias(true);

        match paint.shader {
            Shader::Solid(color) => {
                skia_paint.set_color(skia_safe::Color::from_argb(
                    f32::round(color.a * 255.0) as u8,
                    f32::round(color.r * 255.0) as u8,
                    f32::round(color.g * 255.0) as u8,
                    f32::round(color.b * 255.0) as u8,
                ));
            }
        }

        skia_paint
    }
}

impl Fonts for SkiaFonts {
    fn measure(&mut self, paragraph: &Paragraph, max_width: f32) -> Size {
        let mut min_height = 0.0;

        if let Some((_, style)) = paragraph.sections().next() {
            let typefaces = self.collection.find_typefaces(
                &[&style.font_family],
                Self::build_font_style(style),
            );

            if let Some(typeface) = typefaces.first() {
                let font = skia_safe::Font::new(typeface, style.font_size);
                let (_, metrics) = font.metrics();

                min_height = metrics.descent - metrics.ascent + metrics.leading;
            }
        }

        let mut paragraph = self.build_paragraph(paragraph);
        paragraph.layout(max_width);

        Size {
            width:  paragraph.max_intrinsic_width(),
            height: paragraph.height().max(min_height),
        }
    }

    fn layout(&mut self, paragraph: &Paragraph, max_width: f32) -> Vec<ike::TextLayoutLine> {
        let mut skia = self.build_paragraph(paragraph);
        skia.layout(max_width);

        let mut lines = Vec::new();

        let metrics = skia.get_line_metrics();

        for (i, metric) in metrics.iter().enumerate() {
            let end_index = metric.end_including_newline.saturating_sub(1);

            let has_newline = if paragraph.text.is_char_boundary(end_index) {
                paragraph.text[end_index..].starts_with('\n')
            } else {
                false
            };

            let is_last = i == metrics.len() - 1;

            let (start_index, end_index) = if has_newline {
                if is_last {
                    (
                        metric.start_index + 1,
                        metric.end_including_newline,
                    )
                } else {
                    (metric.start_index, end_index)
                }
            } else {
                (
                    metric.start_index,
                    metric.end_including_newline,
                )
            };

            let mut line = TextLayoutLine {
                ascent: metric.ascent as f32,
                descent: metric.descent as f32,
                left: metric.left as f32,
                width: metric.width as f32,
                height: metric.height as f32,
                baseline: metric.baseline as f32,
                start_index,
                end_index,
                glyphs: Vec::new(),
            };

            for i in metric.start_index..metric.end_index {
                let Some(glyph) = skia.get_glyph_cluster_at(i) else {
                    continue;
                };

                if paragraph.text[glyph.text_range.clone()] == *"\n" {
                    continue;
                }

                // skia doesn't count trailing spaces in the line width
                line.width = line.width.max(glyph.bounds.right);

                let bounds = Rect {
                    min: Point::new(glyph.bounds.left, glyph.bounds.top),
                    max: Point::new(glyph.bounds.right, glyph.bounds.bottom),
                };

                let direction = match glyph.position {
                    skia_safe::textlayout::TextDirection::RTL => TextDirection::Rtl,
                    skia_safe::textlayout::TextDirection::LTR => TextDirection::Ltr,
                };

                line.glyphs.push(GlyphCluster {
                    bounds,
                    start_index: glyph.text_range.start,
                    end_index: glyph.text_range.end,
                    direction,
                });
            }

            lines.push(line);
        }

        lines
    }
}

impl Canvas for SkiaCanvas<'_> {
    fn fonts(&mut self) -> &mut dyn Fonts {
        self.fonts
    }

    fn transform(&mut self, affine: Affine, f: &mut dyn FnMut(&mut dyn Canvas)) {
        let matrix = skia_safe::Matrix::new_all(
            affine.matrix.matrix[0],
            affine.matrix.matrix[1],
            affine.offset.x,
            affine.matrix.matrix[2],
            affine.matrix.matrix[3],
            affine.offset.y,
            0.0,
            0.0,
            1.0,
        );

        self.canvas.save();
        self.canvas.concat(&matrix);

        f(self);

        self.canvas.restore();
    }

    fn draw_rect(&mut self, rect: Rect, radius: CornerRadius, paint: &Paint) {
        let rect = skia_safe::RRect::new_nine_patch(
            skia_safe::Rect::new(
                rect.min.x, rect.min.y, rect.max.x, rect.max.y,
            ),
            radius.top_left,
            radius.top_right,
            radius.bottom_right,
            radius.bottom_left,
        );

        let paint = self.create_paint(paint);

        self.canvas.draw_rrect(rect, &paint);
    }

    fn draw_border(&mut self, rect: Rect, width: BorderWidth, radius: CornerRadius, paint: &Paint) {
        let mut path = skia_safe::Path::new();

        let inner = skia_safe::RRect::new_nine_patch(
            skia_safe::Rect::new(
                rect.min.x + width.left,
                rect.min.y + width.top,
                rect.max.x - width.right,
                rect.max.y - width.bottom,
            ),
            radius.top_left - (width.top + width.left) / 2.0,
            radius.top_right - (width.top + width.right) / 2.0,
            radius.bottom_right - (width.bottom + width.right) / 2.0,
            radius.bottom_left - (width.bottom + width.left) / 2.0,
        );

        let outer = skia_safe::RRect::new_nine_patch(
            skia_safe::Rect::new(
                rect.min.x, rect.min.y, rect.max.x, rect.max.y,
            ),
            radius.top_left,
            radius.top_right,
            radius.bottom_right,
            radius.bottom_left,
        );

        path.add_rrect(
            inner,
            Some((skia_safe::PathDirection::CCW, 0)),
        );
        path.add_rrect(outer, None);

        let paint = self.create_paint(paint);

        self.canvas.draw_path(&path, &paint);
    }

    fn draw_text(&mut self, paragraph: &Paragraph, max_width: f32, offset: Offset) {
        let mut paragraph = self.fonts.build_paragraph(paragraph);
        paragraph.layout(max_width + 1.0);
        paragraph.paint(self.canvas, (offset.x, offset.y));
    }
}
