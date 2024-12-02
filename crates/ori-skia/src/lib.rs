use core::ffi;
use std::{collections::HashMap, mem};

use ori_core::{
    canvas::{Canvas, Color, Curve, CurveSegment, FillRule, Paint, Primitive, Shader},
    image::WeakImage,
    layout::{Affine, Point, Rect, Size, Vector},
    text::{
        FontFamily, FontSource, FontStretch, FontStyle, Fonts, GlyphCluster, Paragraph, TextAlign,
        TextDirection, TextLayoutLine,
    },
};
use skia_safe::{
    font_style::{FontStyle as SkiaFontStyle, Slant, Weight, Width},
    textlayout::{
        FontCollection, Paragraph as SkiaParagraph, ParagraphBuilder, ParagraphStyle,
        TextAlign as SkiaTextAlign, TextDirection as SkiaTextDirection, TextStyle,
        TypefaceFontProvider,
    },
    FontMgr,
};

type Images = HashMap<WeakImage, skia_safe::Image>;
type GlGetIntegerv = unsafe extern "C" fn(u32, *mut i32);

#[allow(dead_code)]
pub struct SkiaFonts {
    collection: FontCollection,
    provider: TypefaceFontProvider,
    manager: FontMgr,
}

impl SkiaFonts {
    pub fn new(default_font: Option<&str>) -> Self {
        let mut collection = FontCollection::new();
        let provider = TypefaceFontProvider::new();
        let manager = FontMgr::new();

        collection.set_dynamic_font_manager(FontMgr::clone(&provider));
        collection.set_default_font_manager(manager.clone(), default_font);

        Self {
            collection,
            provider,
            manager,
        }
    }

    pub fn build_skia_paragraph(&mut self, paragraph: &Paragraph) -> SkiaParagraph {
        let mut style = ParagraphStyle::new();

        let align = match paragraph.align {
            TextAlign::Start => SkiaTextAlign::Left,
            TextAlign::Center => SkiaTextAlign::Center,
            TextAlign::End => SkiaTextAlign::Right,
        };

        style.set_text_align(align);

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

            builder.push_style(&style);
            builder.add_text(text);
            builder.pop();
        }

        builder.build()
    }
}

impl Fonts for SkiaFonts {
    fn load(&mut self, source: FontSource<'_>) {
        let fonts = source.data().unwrap();

        for data in fonts {
            if let Some(typeface) = self.manager.new_from_data(&data, None) {
                self.provider.register_typeface(typeface, None);
            }
        }
    }

    fn layout(&mut self, paragraph: &Paragraph, width: f32) -> Vec<TextLayoutLine> {
        let mut skia_paragraph = self.build_skia_paragraph(paragraph);
        skia_paragraph.layout(width);

        let mut lines = Vec::new();

        for metrics in skia_paragraph.get_line_metrics() {
            let mut line = TextLayoutLine {
                width: metrics.width as f32,
                height: metrics.height as f32,
                baseline: metrics.baseline as f32,
                glyphs: Vec::new(),
            };

            for i in metrics.start_index..metrics.end_index {
                let Some(glyph) = skia_paragraph.get_glyph_cluster_at(i) else {
                    continue;
                };

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
        let mut skia_paragraph = self.build_skia_paragraph(paragraph);
        skia_paragraph.layout(width);

        let width = skia_paragraph.max_intrinsic_width();
        let height = skia_paragraph.height();

        Size::new(width, height)
    }
}

pub struct SkiaRenderer {
    gl_get_integerv: GlGetIntegerv,
    skia: skia_safe::gpu::DirectContext,
    surface: Option<skia_safe::Surface>,
    images: HashMap<WeakImage, skia_safe::Image>,
    width: u32,
    height: u32,
}

impl SkiaRenderer {
    /// # Safety
    /// - `loader` must be a function that returns a valid pointer to a GL function.
    pub unsafe fn new(mut loader: impl FnMut(&str) -> *const ffi::c_void) -> Self {
        let interface = skia_safe::gpu::gl::Interface::new_load_with(&mut loader).unwrap();
        let skia = skia_safe::gpu::direct_contexts::make_gl(interface, None).unwrap();

        let gl_get_integerv =
            mem::transmute::<*const std::ffi::c_void, GlGetIntegerv>(loader("glGetIntegerv"));

        Self {
            gl_get_integerv,
            skia,
            surface: None,
            images: HashMap::new(),
            width: 0,
            height: 0,
        }
    }

    pub fn render(
        &mut self,
        fonts: &mut SkiaFonts,
        canvas: &Canvas,
        color: Color,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) {
        self.update_surface(width, height);

        let skia_canvas = self.surface.as_mut().unwrap().canvas();
        skia_canvas.clear(Self::skia_color(color));

        for primitive in canvas.primitives() {
            let transform = Affine::scale(Vector::all(scale_factor));
            Self::draw_primitive(fonts, &mut self.images, skia_canvas, primitive, transform);
        }

        self.skia.flush_and_submit();
    }

    fn draw_primitive(
        fonts: &mut SkiaFonts,
        images: &mut Images,
        canvas: &skia_safe::Canvas,
        primitive: &Primitive,
        transform: Affine,
    ) {
        match primitive {
            Primitive::Fill { curve, fill, paint } => {
                Self::fill_curve(images, canvas, curve, fill, paint)
            }
            Primitive::Stroke {
                curve,
                stroke,
                paint,
            } => {
                let mut stroked = Curve::new();
                stroked.stroke_curve(curve, *stroke);
                Self::fill_curve(images, canvas, &stroked, &FillRule::NonZero, paint);
            }
            Primitive::Paragraph { paragraph, rect } => {
                let mut skia_paragraph = fonts.build_skia_paragraph(paragraph);
                skia_paragraph.layout(rect.width() + 1.0);

                skia_paragraph.paint(canvas, (rect.min.x, rect.min.y));
            }
            Primitive::Layer {
                primitives,
                transform: layer_transform,
                mask,
                ..
            } => {
                canvas.save();

                let transform = transform * *layer_transform;

                if let Some(mask) = mask {
                    let skia_path = Self::skia_path(&mask.curve);
                    canvas.clip_path(&skia_path, None, true);
                }

                canvas.set_matrix(&Self::skia_matrix(transform).into());

                for primitive in primitives.iter() {
                    Self::draw_primitive(fonts, images, canvas, primitive, transform);
                }

                canvas.restore();
            }
        }
    }

    fn fill_curve(
        images: &mut Images,
        canvas: &skia_safe::Canvas,
        curve: &Curve,
        fill: &FillRule,
        paint: &Paint,
    ) {
        let mut skia_path = Self::skia_path(curve);

        skia_path.set_fill_type(match fill {
            FillRule::NonZero => skia_safe::PathFillType::Winding,
            FillRule::EvenOdd => skia_safe::PathFillType::EvenOdd,
        });

        let color = match paint.shader {
            Shader::Solid(color) => color,
            Shader::Pattern(ref pattern) => pattern.color,
        };

        let mut skia_paint = skia_safe::Paint::new(Self::skia_color_4f(color), None);
        skia_paint.set_anti_alias(true);

        match paint.shader {
            Shader::Pattern(ref pattern) => {
                let weak_image = pattern.image.downgrade();
                let image = images.entry(weak_image).or_insert_with(|| {
                    let image = skia_safe::images::raster_from_data(
                        &skia_safe::ImageInfo::new(
                            skia_safe::ISize::new(
                                pattern.image.width() as i32,
                                pattern.image.height() as i32,
                            ),
                            skia_safe::ColorType::RGBA8888,
                            skia_safe::AlphaType::Unpremul,
                            None,
                        ),
                        skia_safe::Data::new_copy(pattern.image.data()),
                        pattern.image.width() as usize * 4,
                    )
                    .unwrap();

                    image
                });

                let mut transform = pattern.transform;
                transform.translation *= -1.0;

                let shader = skia_safe::shaders::image(
                    image.clone(),
                    (
                        skia_safe::TileMode::default(),
                        skia_safe::TileMode::default(),
                    ),
                    &skia_safe::SamplingOptions::default(),
                    &Self::skia_matrix(transform),
                )
                .unwrap()
                .with_color_filter(
                    skia_safe::color_filters::blend(
                        Self::skia_color(color),
                        skia_safe::BlendMode::Modulate,
                    )
                    .unwrap(),
                );

                skia_paint.set_shader(shader);
            }
            Shader::Solid(_) => {}
        }

        canvas.draw_path(&skia_path, &skia_paint);
    }

    fn skia_path(curve: &Curve) -> skia_safe::Path {
        let mut skia_path = skia_safe::Path::new();

        for segment in curve.iter() {
            match segment {
                CurveSegment::Move(p) => {
                    skia_path.move_to((p.x, p.y));
                }
                CurveSegment::Line(p) => {
                    skia_path.line_to((p.x, p.y));
                }
                CurveSegment::Quad(p0, p1) => {
                    skia_path.quad_to((p0.x, p0.y), (p1.x, p1.y));
                }
                CurveSegment::Cubic(p0, p1, p2) => {
                    skia_path.cubic_to((p0.x, p0.y), (p1.x, p1.y), (p2.x, p2.y));
                }
                CurveSegment::Close => {
                    skia_path.close();
                }
            }
        }

        skia_path
    }

    fn skia_matrix(affine: Affine) -> skia_safe::Matrix {
        let mut matrix = skia_safe::Matrix::new_identity();
        #[rustfmt::skip]
        matrix.set_9(&[
            affine.matrix.x.x,    affine.matrix.y.x, affine.translation.x,
            affine.matrix.x.y,    affine.matrix.y.y, affine.translation.y,
                          0.0,                  0.0,                  1.0,
        ]);
        matrix
    }

    fn skia_color_4f(color: Color) -> skia_safe::Color4f {
        skia_safe::Color4f::new(color.r, color.g, color.b, color.a)
    }

    fn skia_color(color: Color) -> skia_safe::Color {
        skia_safe::Color::from_argb(
            (color.a * 255.0) as u8,
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
        )
    }

    fn update_surface(&mut self, width: u32, height: u32) {
        if self.width != width || self.height != height {
            let mut fboid = 0;
            unsafe { (self.gl_get_integerv)(0x8D40, &mut fboid) };

            let fbinfo = skia_safe::gpu::gl::FramebufferInfo {
                fboid: fboid as u32,
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
                ..Default::default()
            };

            let sample_count = 4;
            let stencil_bits = 0;

            let backend_render_target = skia_safe::gpu::backend_render_targets::make_gl(
                (width as i32, height as i32),
                sample_count,
                stencil_bits,
                fbinfo,
            );

            let surface = skia_safe::gpu::surfaces::wrap_backend_render_target(
                &mut self.skia,
                &backend_render_target,
                skia_safe::gpu::SurfaceOrigin::BottomLeft,
                skia_safe::ColorType::RGBA8888,
                None,
                None,
            )
            .unwrap();

            self.surface = Some(surface);
        }
    }
}
