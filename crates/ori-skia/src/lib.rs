use core::ffi;
use std::{collections::HashMap, mem};

use ori_core::{
    canvas::{Canvas, Color, Curve, CurveSegment, FillRule, Paint, Primitive, Shader},
    image::WeakImage,
    layout::{Affine, Vector},
};

type Images = HashMap<WeakImage, skia_safe::Image>;
type GlGetIntegerv = unsafe extern "C" fn(u32, *mut i32);

pub struct SkiaRenderer {
    gl_get_integerv: GlGetIntegerv,
    skia: skia_safe::gpu::DirectContext,
    surface: Option<skia_safe::Surface>,
    images: HashMap<WeakImage, skia_safe::Image>,
    width: u32,
    height: u32,
}

impl SkiaRenderer {
    pub fn new(mut loader: impl FnMut(&str) -> *const ffi::c_void) -> Self {
        let interface = skia_safe::gpu::gl::Interface::new_load_with(&mut loader).unwrap();
        let skia = skia_safe::gpu::direct_contexts::make_gl(interface, None).unwrap();

        let gl_get_integerv = unsafe {
            mem::transmute::<*const std::ffi::c_void, GlGetIntegerv>(loader("glGetIntegerv"))
        };

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
            Self::draw_primitive(&mut self.images, skia_canvas, primitive, transform);
        }

        self.skia.flush_and_submit();
    }

    fn draw_primitive(
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
                    Self::draw_primitive(images, canvas, primitive, transform);
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
