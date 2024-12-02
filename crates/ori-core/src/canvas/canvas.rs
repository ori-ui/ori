use std::{
    collections::BTreeMap,
    hash::{Hash, Hasher},
    mem,
    sync::Arc,
};

use crate::{
    image::Image,
    layout::{Affine, Point, Rect, Vector},
    text::Paragraph,
    view::ViewId,
};

use super::{Color, Curve, Stroke};

/// A pattern that can be used to fill a shape.
#[derive(Clone, Debug, PartialEq)]
pub struct Pattern {
    /// The image of the pattern.
    pub image: Image,

    /// The transformation of the pattern.
    pub transform: Affine,

    /// The color of the pattern.
    pub color: Color,
}

impl Hash for Pattern {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.image.hash(state);
        self.transform.hash(state);
        self.color.hash(state);
    }
}

impl From<Image> for Pattern {
    fn from(value: Image) -> Self {
        Self {
            image: value,
            transform: Affine::IDENTITY,
            color: Color::WHITE,
        }
    }
}

/// Ways to fill a shape.
#[derive(Clone, Debug, PartialEq, Hash)]
pub enum Shader {
    /// A solid color.
    Solid(Color),

    /// A pattern.
    Pattern(Pattern),
}

/// Ways to blend two colors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// Replaces the destination with zero.
    Clear,

    /// Replaces the destination with the source.
    Source,

    /// Preserves the destination.
    Destination,

    /// Source over destination.
    SourceOver,

    /// Destination over source.
    DestinationOver,
}

/// Ways to anti-alias a shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AntiAlias {
    /// No anti-aliasing.
    None,

    /// Fast anti-aliasing.
    Fast,

    /// Anti-aliasing.
    Full,
}

/// A paint that can be used to fill or stroke a shape.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Paint {
    /// The shader of the paint.
    pub shader: Shader,

    /// The blend mode of the paint.
    pub blend: BlendMode,

    /// Whether the paint should be anti-aliased.
    pub anti_alias: AntiAlias,
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            shader: Shader::Solid(Color::BLACK),
            blend: BlendMode::SourceOver,
            anti_alias: AntiAlias::Fast,
        }
    }
}

impl From<Color> for Paint {
    fn from(value: Color) -> Self {
        Self {
            shader: Shader::Solid(value),
            ..Default::default()
        }
    }
}

impl From<Image> for Paint {
    fn from(value: Image) -> Self {
        Self {
            shader: Shader::Pattern(Pattern::from(value)),
            ..Default::default()
        }
    }
}

impl From<Pattern> for Paint {
    fn from(value: Pattern) -> Self {
        Self {
            shader: Shader::Pattern(value),
            ..Default::default()
        }
    }
}

/// Rule determining if a point is inside a shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FillRule {
    /// A point is inside the shape if a ray from the point crosses a non-zero sum of signed edge
    /// crossings.
    NonZero,

    /// A point is inside the shape if a ray from the point crosses an odd number of edges.
    EvenOdd,
}

/// A mask that can be used to clip a layer.
#[derive(Clone, Debug, PartialEq)]
pub struct Mask {
    /// The curve of the mask.
    pub curve: Arc<Curve>,

    /// The fill rule of the mask.
    pub fill: FillRule,
}

impl Mask {
    /// Create a new mask.
    pub fn new(curve: impl Into<Arc<Curve>>, fill: FillRule) -> Self {
        Self {
            curve: curve.into(),
            fill,
        }
    }
}

impl From<Rect> for Mask {
    fn from(value: Rect) -> Self {
        Self::new(Curve::rect(value), FillRule::NonZero)
    }
}

/// A primitive that can be drawn on a canvas.
#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    /// A filled curve.
    Fill {
        /// The curve to draw.
        curve: Arc<Curve>,

        /// The fill rule of the curve.
        fill: FillRule,

        /// The paint to fill the curve with.
        paint: Paint,
    },

    /// A stroked curve.
    Stroke {
        /// The curve to draw.
        curve: Arc<Curve>,

        /// The stroke properties of the curve.
        stroke: Stroke,

        /// The paint to stroke the curve with.
        paint: Paint,
    },

    /// A paragraph on rich text.
    Paragraph {
        /// The paragraph to draw.
        paragraph: Paragraph,

        /// The bounding rectangle of the paragraph.
        bounds: Rect,

        /// The rectangle to draw the paragraph in.
        rect: Rect,
    },

    /// A layer that can be transformed and masked.
    Layer {
        /// The primitives of the layer.
        primitives: Arc<Vec<Primitive>>,

        /// The transformation of the layer.
        transform: Affine,

        /// The mask of the layer.
        mask: Option<Mask>,

        /// The view of the layer.
        view: Option<ViewId>,
    },
}

impl Primitive {
    /// Count the number of primitives.
    pub fn count(&self) -> usize {
        match self {
            Primitive::Fill { .. } | Primitive::Stroke { .. } | Primitive::Paragraph { .. } => 1,
            Primitive::Layer { primitives, .. } => primitives.iter().map(Self::count).sum(),
        }
    }
}

/// A canvas that can be drawn on.
#[derive(Clone, Debug, PartialEq)]
pub struct Canvas {
    overlays: BTreeMap<i32, Arc<Vec<Primitive>>>,
    primitives: Arc<Vec<Primitive>>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Canvas {
    /// Create a new canvas.
    pub fn new() -> Self {
        Self {
            overlays: BTreeMap::new(),
            primitives: Arc::new(Vec::new()),
        }
    }

    /// Get the primitives of the canvas.
    pub fn primitives(&self) -> impl Iterator<Item = &Primitive> + '_ {
        let overlays = self.overlays.values().flat_map(|p| p.iter());
        self.primitives.iter().chain(overlays)
    }

    /// Clear the canvas.
    pub fn clear(&mut self) {
        self.overlays.clear();
        Arc::make_mut(&mut self.primitives).clear();
    }

    /// Draw a rectangle.
    pub fn rect(&mut self, rect: Rect, paint: impl Into<Paint>) {
        let curve = Curve::rect(rect);
        self.fill(curve.clone(), FillRule::NonZero, paint);
    }

    /// Draw a trigger rectangle.
    pub fn trigger(&mut self, rect: Rect, view: ViewId) {
        self.hoverable(view, |canvas| {
            let curve = Curve::rect(rect);
            canvas.fill(
                curve,
                FillRule::NonZero,
                Paint {
                    shader: Shader::Solid(Color::TRANSPARENT),
                    blend: BlendMode::Destination,
                    anti_alias: AntiAlias::None,
                },
            );
        });
    }

    /// Fill a curve.
    pub fn fill(&mut self, curve: impl Into<Arc<Curve>>, fill: FillRule, paint: impl Into<Paint>) {
        let primitives = Arc::make_mut(&mut self.primitives);
        primitives.push(Primitive::Fill {
            curve: curve.into(),
            fill,
            paint: paint.into(),
        });
    }

    /// Stroke a curve.
    pub fn stroke(
        &mut self,
        curve: impl Into<Arc<Curve>>,
        stroke: impl Into<Stroke>,
        paint: impl Into<Paint>,
    ) {
        let primitives = Arc::make_mut(&mut self.primitives);
        primitives.push(Primitive::Stroke {
            curve: curve.into(),
            stroke: stroke.into(),
            paint: paint.into(),
        });
    }

    /// Draw a paragraph.
    pub fn text(&mut self, paragraph: Paragraph, rect: Rect, bounds: Rect) {
        let primitives = Arc::make_mut(&mut self.primitives);

        primitives.push(Primitive::Paragraph {
            paragraph,
            bounds,
            rect,
        });
    }

    /// Draw a canvas.
    pub fn draw_canvas(&mut self, canvas: Canvas) {
        self.layer(Affine::IDENTITY, None, None, |ca| *ca = canvas);
    }

    /// Draw an overlay.
    pub fn overlay<T>(&mut self, index: i32, f: impl FnOnce(&mut Self) -> T) -> T {
        let mut overlay = Canvas::new();

        let result = f(&mut overlay);

        for (i, mut others) in overlay.overlays {
            let others = mem::take(Arc::make_mut(&mut others));
            let primitives = Arc::make_mut(self.overlays.entry(i).or_default());
            primitives.extend(others);
        }

        let other = mem::take(Arc::make_mut(&mut overlay.primitives));
        let primitives = Arc::make_mut(self.overlays.entry(index).or_default());
        primitives.extend(other);

        result
    }

    /// Draw a layer.
    pub fn layer<T>(
        &mut self,
        transform: Affine,
        mask: Option<Mask>,
        view: Option<ViewId>,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let mut layer = Canvas::new();

        let result = f(&mut layer);

        for (i, mut other) in layer.overlays {
            let other = mem::take(Arc::make_mut(&mut other));
            let primitives = Arc::make_mut(self.overlays.entry(i).or_default());
            primitives.extend(other);
        }

        let primitives = Arc::make_mut(&mut self.primitives);
        primitives.push(Primitive::Layer {
            primitives: layer.primitives,
            transform,
            mask,
            view,
        });

        result
    }

    /// Draw a layer with a transformation.
    pub fn transformed<T>(&mut self, transform: Affine, f: impl FnOnce(&mut Self) -> T) -> T {
        self.layer(transform, None, None, f)
    }

    /// Draw a layer with a translation.
    pub fn translated<T>(&mut self, translation: Vector, f: impl FnOnce(&mut Self) -> T) -> T {
        self.transformed(Affine::translate(translation), f)
    }

    /// Draw a layer with a rotation.
    pub fn rotated<T>(&mut self, angle: f32, f: impl FnOnce(&mut Self) -> T) -> T {
        self.transformed(Affine::rotate(angle), f)
    }

    /// Draw a layer with a scale.
    pub fn scaled<T>(&mut self, scale: Vector, f: impl FnOnce(&mut Self) -> T) -> T {
        self.transformed(Affine::scale(scale), f)
    }

    /// Draw a layer with a mask.
    pub fn masked<T>(&mut self, mask: Mask, f: impl FnOnce(&mut Self) -> T) -> T {
        self.layer(Affine::IDENTITY, Some(mask), None, f)
    }

    /// Draw a layer with a view.
    pub fn hoverable<T>(&mut self, view: ViewId, f: impl FnOnce(&mut Self) -> T) -> T {
        self.layer(Affine::IDENTITY, None, Some(view), f)
    }

    /// Get the view at a point.
    pub fn view_at(&self, point: Point) -> Option<ViewId> {
        fn recurse(primitives: &[Primitive], view: Option<ViewId>, point: Point) -> Option<ViewId> {
            for primitive in primitives.iter().rev() {
                match primitive {
                    Primitive::Fill { curve, fill, .. } => {
                        if view.is_some() && curve.contains(point, *fill) {
                            return view;
                        }
                    }
                    Primitive::Stroke { .. } => {}
                    Primitive::Paragraph { rect, .. } => {
                        if rect.contains(point) {
                            return view;
                        }
                    }
                    Primitive::Layer {
                        primitives,
                        transform,
                        mask,
                        view: layer_view,
                    } => {
                        let point = transform.inverse() * point;

                        if let Some(mask) = mask {
                            if !mask.curve.contains(point, mask.fill) {
                                continue;
                            }
                        }

                        let view = match layer_view {
                            Some(view) => recurse(primitives, Some(*view), point),
                            None => recurse(primitives, view, point),
                        };

                        if view.is_some() {
                            return view;
                        }
                    }
                }
            }

            None
        }

        for primitives in self.overlays.values().rev() {
            if let Some(view) = recurse(primitives, None, point) {
                return Some(view);
            }
        }

        recurse(&self.primitives, None, point)
    }
}
