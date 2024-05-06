use ori_core::{
    canvas::{
        BlendMode, Canvas, Curve, CurveSegment, FillRule, LineCap, LineJoin, Paint, Primitive,
        Shader, Stroke,
    },
    layout::{Affine, Rect, Vector},
};
use tiny_skia::PixmapMut;

pub fn render_canvas(pixmap: &mut PixmapMut<'_>, canvas: &Canvas, offset: Vector) {
    let transform = tiny_skia::Transform::from_translate(-offset.x, -offset.y);

    render_primitives(pixmap, canvas.primitives(), transform, None);
}

fn render_primitives(
    pixmap: &mut PixmapMut<'_>,
    primitives: &[Primitive],
    transform: tiny_skia::Transform,
    mask: Option<&tiny_skia::Mask>,
) {
    for primitive in primitives {
        render_primitive(pixmap, primitive, transform, mask);
    }
}

fn render_primitive(
    pixmap: &mut PixmapMut<'_>,
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

fn map_shader(shader: &Shader) -> tiny_skia::Shader<'_> {
    match shader {
        Shader::Solid(color) => {
            let color = tiny_skia::Color::from_rgba(
                color.r.clamp(0.0, 1.0),
                color.g.clamp(0.0, 1.0),
                color.b.clamp(0.0, 1.0),
                color.a.clamp(0.0, 1.0),
            )
            .unwrap();

            tiny_skia::Shader::SolidColor(color)
        }
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
        FillRule::NonZero => tiny_skia::FillRule::Winding,
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
