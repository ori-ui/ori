use std::collections::HashMap;

use ike::{
    Affine, BorderWidth, Canvas, CornerRadius, FontStretch, FontStyle, GlyphCluster, Offset, Paint,
    Painter, Paragraph, Point, Rect, Shader, Size, Svg, TextDirection, TextLayoutLine, TextStyle,
    TextWrap, WeakSvg,
};

pub(crate) struct SkiaPainter {
    pub(crate) provider: skia_safe::textlayout::TypefaceFontProvider,
    pub(crate) manager:  skia_safe::FontMgr,
    pub(crate) fonts:    skia_safe::textlayout::FontCollection,
    pub(crate) svgs:     HashMap<WeakSvg, Option<skia_safe::svg::Dom>>,
}

impl Default for SkiaPainter {
    fn default() -> Self {
        Self::new()
    }
}

impl SkiaPainter {
    pub(crate) fn new() -> Self {
        let provider = skia_safe::textlayout::TypefaceFontProvider::new();
        let manager = skia_safe::FontMgr::new();
        let mut fonts = skia_safe::textlayout::FontCollection::new();
        fonts.set_dynamic_font_manager(skia_safe::FontMgr::clone(&provider));
        fonts.set_default_font_manager(manager.clone(), None);

        Self {
            provider,
            manager,
            fonts,
            svgs: HashMap::new(),
        }
    }

    pub(crate) fn cleanup(&mut self) {
        self.svgs.retain(|k, _| k.strong_count() > 0);
    }

    pub(crate) fn load_font(&mut self, bytes: &[u8], alias: Option<&str>) {
        if let Some(typeface) = self.manager.new_from_data(bytes, None) {
            self.provider.register_typeface(typeface, alias);
        } else {
            tracing::warn!("loading font failed");
        }
    }

    fn create_svg(&mut self, svg: &Svg) -> Option<skia_safe::svg::Dom> {
        let weak = Svg::downgrade(svg);

        self.svgs
            .entry(weak)
            .or_insert_with(|| {
                let dom = skia_safe::svg::Dom::from_bytes(
                    svg.bytes(),
                    skia_safe::FontMgr::default(),
                )
                .ok()?;

                let mut svg = dom.root();

                if svg.intrinsic_size().is_zero() {
                    svg.set_height(skia_safe::svg::Length::new(
                        1.0,
                        skia_safe::svg::LengthUnit::PX,
                    ));
                    svg.set_width(skia_safe::svg::Length::new(
                        1.0,
                        skia_safe::svg::LengthUnit::PX,
                    ));
                }

                Some(dom)
            })
            .clone()
    }

    fn crate_font_style(style: &TextStyle) -> skia_safe::FontStyle {
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

    fn crate_paragraph(&mut self, paragraph: &Paragraph) -> skia_safe::textlayout::Paragraph {
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

        let mut builder = skia_safe::textlayout::ParagraphBuilder::new(&style, &self.fonts);

        for (text, style) in paragraph.sections() {
            let mut skia_style = skia_safe::textlayout::TextStyle::new();

            skia_style.set_font_size(style.font_size);
            skia_style.set_font_families(&[&style.font_family]);
            skia_style.set_font_style(Self::crate_font_style(style));
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
    pub(crate) canvas:  &'a skia_safe::Canvas,
    pub(crate) painter: &'a mut SkiaPainter,
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

        let blend = match paint.blend {
            ike::BlendMode::Clear => skia_safe::BlendMode::Clear,
            ike::BlendMode::Src => skia_safe::BlendMode::Src,
            ike::BlendMode::Dst => skia_safe::BlendMode::Dst,
            ike::BlendMode::SrcOver => skia_safe::BlendMode::SrcOver,
            ike::BlendMode::DstOver => skia_safe::BlendMode::DstOver,
            ike::BlendMode::SrcATop => skia_safe::BlendMode::SrcATop,
            ike::BlendMode::DstATop => skia_safe::BlendMode::DstATop,
        };

        skia_paint.set_blend_mode(blend);

        skia_paint
    }
}

impl Painter for SkiaPainter {
    fn measure_svg(&mut self, svg: &Svg) -> Size {
        if let Some(skia_dom) = self.create_svg(svg) {
            let size = skia_dom.root().intrinsic_size();
            Size::new(size.width, size.height)
        } else {
            Size::ZERO
        }
    }

    fn measure_text(&mut self, paragraph: &Paragraph, max_width: f32) -> Size {
        let mut min_height = 0.0;

        if let Some((_, style)) = paragraph.sections().next() {
            let typefaces = self.fonts.find_typefaces(
                &[&style.font_family],
                Self::crate_font_style(style),
            );

            if let Some(typeface) = typefaces.first() {
                let font = skia_safe::Font::new(typeface, style.font_size);
                let (_, metrics) = font.metrics();

                min_height = metrics.descent - metrics.ascent + metrics.leading;
            }
        }

        let mut paragraph = self.crate_paragraph(paragraph);
        paragraph.layout(max_width);

        Size {
            width:  paragraph.max_intrinsic_width(),
            height: paragraph.height().max(min_height),
        }
    }

    fn layout_text(&mut self, paragraph: &Paragraph, max_width: f32) -> Vec<ike::TextLayoutLine> {
        let mut skia = self.crate_paragraph(paragraph);
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
    fn painter(&mut self) -> &mut dyn Painter {
        self.painter
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

    fn layer(&mut self, f: &mut dyn FnMut(&mut dyn Canvas)) {
        self.canvas.save_layer_alpha_f(None, 1.0);
        f(self);
        self.canvas.restore();
    }

    fn fill(&mut self, paint: &Paint) {
        let paint = self.create_paint(paint);
        self.canvas.draw_paint(&paint);
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
        let mut paragraph = self.painter.crate_paragraph(paragraph);
        paragraph.layout(max_width + 1.0);
        paragraph.paint(self.canvas, (offset.x, offset.y));
    }

    fn draw_svg(&mut self, svg: &Svg) {
        if let Some(skia_dom) = self.painter.create_svg(svg) {
            skia_dom.render(self.canvas);
        }
    }
}
