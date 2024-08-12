//! A tiny-skia renderer for Ori.

#![deny(missing_docs)]

use ori_core::{
    canvas::{
        BlendMode, Canvas, CanvasDiff, Color, Curve, CurveSegment, FillRule, LineCap, LineJoin,
        Paint, Primitive, Shader, Stroke,
    },
    layout::{Affine, Rect, Vector},
};

/// A buffer that can be used to render a canvas.
pub enum Buffer<'a> {
    /// A buffer with RGBA8 pixel format.
    Rgba8(&'a mut [u8]),

    /// A buffer with ARGB8 pixel format.
    Argb8(&'a mut [u8]),
}

impl Buffer<'_> {
    /// Get the number of bytes in the buffer.
    pub fn len(&self) -> usize {
        match self {
            Buffer::Rgba8(data) => data.len(),
            Buffer::Argb8(data) => data.len(),
        }
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A renderer that uses TinySkia to render a canvas.
pub struct TinySkiaRenderer {
    width: u32,
    height: u32,
    data: Vec<u8>,
    scratch: Vec<u8>,
    previous_canvas: Option<Canvas>,
    clear_color: Option<Color>,
    diff: CanvasDiff,
}

impl TinySkiaRenderer {
    const fn data_len(width: u32, height: u32) -> usize {
        (width * height * 4) as usize
    }

    /// Create a new renderer.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; Self::data_len(width, height)],
            scratch: Vec::new(),
            previous_canvas: None,
            clear_color: None,
            diff: CanvasDiff::new(),
        }
    }

    /// Resize the renderer.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        self.data.resize(Self::data_len(width, height), 0);
        self.previous_canvas = None;
    }

    /// Render the canvas to the buffer.
    pub fn render(&mut self, buffer: &mut Buffer<'_>, canvas: &Canvas, clear_color: Color) {
        match self.previous_canvas {
            Some(ref mut previous_canvas) if self.clear_color == Some(clear_color) => {
                self.diff.update(canvas, previous_canvas);
                self.diff.simplify();

                let mut pixmap = as_pixmap_mut(&mut self.data, self.width, self.height);

                for rect in self.diff.rects() {
                    if rect.area() < 1e-6 {
                        continue;
                    }

                    let min = rect.min.floor() - 1.0;
                    let max = rect.max.ceil() + 1.0;

                    let mut x = min.x as i32;
                    let mut y = min.y as i32;
                    let mut w = (max.x - min.x) as u32;
                    let mut h = (max.y - min.y) as u32;

                    x = x.clamp(0, self.width as i32);
                    y = y.clamp(0, self.height as i32);
                    w = w.clamp(0, self.width - x as u32);
                    h = h.clamp(0, self.height - y as u32);

                    // really make sure the rect is not empty
                    if w == 0 || h == 0 {
                        continue;
                    }

                    if self.scratch.len() < Self::data_len(w, h) {
                        self.scratch.resize(Self::data_len(w, h), 0);
                    }

                    let mut scratch = as_pixmap_mut(&mut self.scratch, w, h);

                    scratch.fill(map_color(clear_color));

                    render_canvas(&mut scratch, canvas, Vector::new(x as f32, y as f32));

                    for j in 0..h as usize {
                        let src = j * w as usize;
                        let dst = (y as usize + j) * self.width as usize + x as usize;

                        for i in 0..w as usize {
                            unsafe {
                                *pixmap.pixels_mut().get_unchecked_mut(dst + i) =
                                    *scratch.pixels_mut().get_unchecked(src + i);
                            }
                        }
                    }
                }

                write_buffer(buffer, &self.data);

                self.previous_canvas = Some(canvas.clone());
            }
            _ => {
                let mut pixmap = as_pixmap_mut(&mut self.data, self.width, self.height);
                pixmap.fill(map_color(clear_color));
                render_canvas(&mut pixmap, canvas, Vector::ZERO);
                write_buffer(buffer, &self.data);

                self.previous_canvas = Some(canvas.clone());
                self.clear_color = Some(clear_color);
            }
        }
    }
}

fn write_buffer(buffer: &mut Buffer<'_>, data: &[u8]) {
    assert_eq!(buffer.len(), data.len());

    match buffer {
        Buffer::Rgba8(dst) => dst.copy_from_slice(data),
        Buffer::Argb8(dst) => {
            for i in 0..dst.len() / 4 {
                let idx = i * 4;

                // SAFETY: the assertion above
                unsafe {
                    *dst.get_unchecked_mut(idx) = *data.get_unchecked(idx + 2);
                    *dst.get_unchecked_mut(idx + 1) = *data.get_unchecked(idx + 1);
                    *dst.get_unchecked_mut(idx + 2) = *data.get_unchecked(idx);
                    *dst.get_unchecked_mut(idx + 3) = *data.get_unchecked(idx + 3);
                }
            }
        }
    }
}

fn as_pixmap_mut(data: &mut [u8], width: u32, height: u32) -> tiny_skia::PixmapMut<'_> {
    debug_assert!(width > 0 && height > 0);
    tiny_skia::PixmapMut::from_bytes(data, width, height).unwrap()
}

fn render_canvas(pixmap: &mut tiny_skia::PixmapMut<'_>, canvas: &Canvas, offset: Vector) {
    let transform = tiny_skia::Transform::from_translate(-offset.x, -offset.y);

    render_primitives(pixmap, canvas.primitives(), transform, None);
}

fn render_primitives<'a>(
    pixmap: &mut tiny_skia::PixmapMut<'_>,
    primitives: impl IntoIterator<Item = &'a Primitive>,
    transform: tiny_skia::Transform,
    mask: Option<&tiny_skia::Mask>,
) {
    for primitive in primitives {
        render_primitive(pixmap, primitive, transform, mask);
    }
}

fn render_primitive(
    pixmap: &mut tiny_skia::PixmapMut<'_>,
    primitive: &Primitive,
    transform: tiny_skia::Transform,
    mask: Option<&tiny_skia::Mask>,
) {
    match primitive {
        Primitive::Rect { rect, paint } => {
            pixmap.fill_rect(map_rect(rect), &map_paint(paint), transform, mask)
        }
        Primitive::Fill { curve, fill, paint } => {
            let curve = match map_curve(curve) {
                Some(curve) => curve,
                None => return,
            };

            pixmap.fill_path(
                &curve,
                &map_paint(paint),
                map_fill_rule(fill),
                transform,
                mask,
            )
        }
        Primitive::Stroke {
            curve,
            stroke,
            paint,
        } => {
            let curve = match map_curve(curve) {
                Some(curve) => curve,
                None => return,
            };

            pixmap.stroke_path(
                &curve,
                &map_paint(paint),
                &map_stroke(stroke),
                transform,
                mask,
            )
        }
        Primitive::Image { point, image } => {
            if !image.is_empty() {
                let x = point.x.round() as i32;
                let y = point.y.round() as i32;

                let src =
                    tiny_skia::PixmapRef::from_bytes(image.data(), image.width(), image.height())
                        .unwrap();

                pixmap.draw_pixmap(x, y, src, &Default::default(), transform, mask);
            }
        }
        Primitive::Layer {
            primitives,
            transform: layer_transform,
            mask: layer_mask,
            ..
        } => {
            let transform = transform.pre_concat(map_transform(layer_transform));

            match layer_mask {
                Some(layer_mask) => {
                    if layer_mask.curve.bounds().area().abs() < 1e-6 {
                        return;
                    }

                    let mut mask = match mask {
                        Some(mask) => mask.clone(),
                        None => tiny_skia::Mask::new(pixmap.width(), pixmap.height()).unwrap(),
                    };

                    let curve = match map_curve(&layer_mask.curve) {
                        Some(curve) => curve,
                        None => return,
                    };

                    mask.fill_path(&curve, map_fill_rule(&layer_mask.fill), true, transform);

                    render_primitives(pixmap, primitives, transform, Some(&mask));
                }
                None => {
                    render_primitives(pixmap, primitives, transform, mask);
                }
            }
        }
    }
}

fn map_paint(paint: &Paint) -> tiny_skia::Paint<'_> {
    tiny_skia::Paint {
        shader: map_shader(&paint.shader),
        blend_mode: map_blend_mode(&paint.blend),
        anti_alias: paint.anti_alias,
        force_hq_pipeline: false,
    }
}

fn map_color(color: Color) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba(
        color.r.clamp(0.0, 1.0),
        color.g.clamp(0.0, 1.0),
        color.b.clamp(0.0, 1.0),
        color.a.clamp(0.0, 1.0),
    )
    .unwrap()
}

fn map_shader(shader: &Shader) -> tiny_skia::Shader<'_> {
    match shader {
        Shader::Solid(color) => tiny_skia::Shader::SolidColor(map_color(*color)),
        Shader::Pattern(pattern) => {
            let pixmap = tiny_skia::PixmapRef::from_bytes(
                pattern.image.data(),
                pattern.image.width(),
                pattern.image.height(),
            )
            .unwrap();

            tiny_skia::Pattern::new(
                pixmap,
                tiny_skia::SpreadMode::Pad,
                tiny_skia::FilterQuality::Bilinear,
                pattern.opacity,
                map_transform(&pattern.transform),
            )
        }
    }
}

fn map_blend_mode(blend: &BlendMode) -> tiny_skia::BlendMode {
    match blend {
        BlendMode::Clear => tiny_skia::BlendMode::Clear,
        BlendMode::Source => tiny_skia::BlendMode::Source,
        BlendMode::Destination => tiny_skia::BlendMode::Destination,
        BlendMode::SourceOver => tiny_skia::BlendMode::SourceOver,
        BlendMode::DestinationOver => tiny_skia::BlendMode::DestinationOver,
    }
}

fn map_rect(rect: &Rect) -> tiny_skia::Rect {
    tiny_skia::Rect::from_xywh(
        rect.min.x,
        rect.min.y,
        rect.size().width,
        rect.size().height,
    )
    .unwrap()
}

fn map_transform(transform: &Affine) -> tiny_skia::Transform {
    tiny_skia::Transform::from_row(
        transform.matrix.x.x,
        transform.matrix.x.y,
        transform.matrix.y.x,
        transform.matrix.y.y,
        transform.translation.x,
        transform.translation.y,
    )
}

fn map_fill_rule(fill: &FillRule) -> tiny_skia::FillRule {
    match fill {
        FillRule::Winding => tiny_skia::FillRule::Winding,
        FillRule::EvenOdd => tiny_skia::FillRule::EvenOdd,
    }
}

fn map_curve(curve: &Curve) -> Option<tiny_skia::Path> {
    let mut path = tiny_skia::PathBuilder::new();

    for segment in curve {
        match segment {
            CurveSegment::Move(p) => path.move_to(p.x, p.y),
            CurveSegment::Line(p) => path.line_to(p.x, p.y),
            CurveSegment::Quad(a, p) => path.quad_to(a.x, a.y, p.x, p.y),
            CurveSegment::Cubic(a, b, p) => path.cubic_to(a.x, a.y, b.x, b.y, p.x, p.y),
            CurveSegment::Close => path.close(),
        }
    }

    path.finish()
}

fn map_stroke(stroke: &Stroke) -> tiny_skia::Stroke {
    tiny_skia::Stroke {
        width: stroke.width,
        miter_limit: stroke.miter,
        line_cap: map_line_cap(&stroke.cap),
        line_join: map_line_join(&stroke.join),
        dash: None,
    }
}

fn map_line_cap(cap: &LineCap) -> tiny_skia::LineCap {
    match cap {
        LineCap::Butt => tiny_skia::LineCap::Butt,
        LineCap::Round => tiny_skia::LineCap::Round,
        LineCap::Square => tiny_skia::LineCap::Square,
    }
}

fn map_line_join(join: &LineJoin) -> tiny_skia::LineJoin {
    match join {
        LineJoin::Miter => tiny_skia::LineJoin::Miter,
        LineJoin::Round => tiny_skia::LineJoin::Round,
        LineJoin::Bevel => tiny_skia::LineJoin::Bevel,
    }
}
