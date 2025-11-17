use ike::{Affine, Canvas, CornerRadius, Fonts, Offset, Paint, Paragraph, Rect, Shader, Size};

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

    pub(crate) fn build_paragraph(
        &mut self,
        paragraph: &Paragraph,
    ) -> skia_safe::textlayout::Paragraph {
        let mut style = skia_safe::textlayout::ParagraphStyle::new();

        style.set_height(16.0);
        style.set_text_align(skia_safe::textlayout::TextAlign::Start);

        let mut builder = skia_safe::textlayout::ParagraphBuilder::new(&style, &self.collection);

        for (text, style) in paragraph.sections() {
            let mut skia_style = skia_safe::textlayout::TextStyle::new();

            skia_style.set_font_size(style.font_size);
            skia_style.set_font_families(&[&style.font_family]);
            skia_style.set_font_style(skia_safe::FontStyle::normal());
            skia_style.set_color(skia_safe::Color::BLACK);

            builder.push_style(&skia_style);
            builder.add_text(text);
            builder.pop();
        }

        builder.build()
    }
}

pub(crate) struct SkiaCanvas<'a> {
    pub(crate) canvas: &'a skia_safe::Canvas,
    pub(crate) fonts: &'a mut SkiaFonts,
}

impl<'a> SkiaCanvas<'a> {
    pub(crate) fn new(canvas: &'a skia_safe::Canvas, fonts: &'a mut SkiaFonts) -> Self {
        Self { canvas, fonts }
    }

    fn create_paint(&self, paint: &Paint) -> skia_safe::Paint {
        let mut skia_paint = skia_safe::Paint::default();

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
        let mut paragraph = self.build_paragraph(paragraph);
        paragraph.layout(max_width);

        Size {
            width: paragraph.max_intrinsic_width(),
            height: paragraph.height(),
        }
    }
}

impl Canvas for SkiaCanvas<'_> {
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

    fn draw_rect(&mut self, rect: Rect, corners: CornerRadius, paint: &Paint) {
        let rect = skia_safe::RRect::new_nine_patch(
            skia_safe::Rect::new(
                rect.min.x, rect.min.y, rect.max.x, rect.max.y,
            ),
            corners.top_left,
            corners.top_right,
            corners.bottom_right,
            corners.bottom_left,
        );

        let paint = self.create_paint(paint);

        self.canvas.draw_rrect(rect, &paint);
    }

    fn draw_text(&mut self, paragraph: &Paragraph, max_width: f32, offset: Offset) {
        let mut paragraph = self.fonts.build_paragraph(paragraph);
        paragraph.layout(max_width + 1.0);
        paragraph.paint(self.canvas, (offset.x, offset.y));
    }
}
