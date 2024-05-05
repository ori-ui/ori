use std::f32::consts::PI;

use crate::layout::{Affine, Point, Rect, Vector};

use super::{BorderRadius, FillRule};

/// A verb that describes the type of curve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CurveVerb {
    /// A move verb.
    Move,

    /// A line verb.
    Line,

    /// A quad verb.
    Quad,

    /// A cubic verb.
    Cubic,

    /// A close verb.
    Close,
}

impl CurveVerb {
    /// Get the number of points for the verb.
    pub fn points(&self) -> usize {
        match self {
            CurveVerb::Move => 1,
            CurveVerb::Line => 1,
            CurveVerb::Quad => 2,
            CurveVerb::Cubic => 3,
            CurveVerb::Close => 0,
        }
    }
}

/// A segment of a curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CurveSegment {
    /// Move to a point.
    Move(Point),

    /// Line to a point.
    Line(Point),

    /// Quadratic bezier curve to a point.
    Quad(Point, Point),

    /// Cubic bezier curve to a point.
    Cubic(Point, Point, Point),

    /// Close the curve.
    Close,
}

/// A bezier curve.
#[derive(Clone, Debug, PartialEq)]
pub struct Curve {
    verbs: Vec<CurveVerb>,
    points: Vec<Point>,
    bounds: Rect,
}

impl Default for Curve {
    fn default() -> Self {
        Self::new()
    }
}

impl Curve {
    /// Create a new curve.
    pub fn new() -> Self {
        Self {
            verbs: Vec::new(),
            points: Vec::new(),
            bounds: Rect::ZERO,
        }
    }

    /// Create a curve from a rectangle.
    pub fn rect(rect: Rect) -> Self {
        let mut curve = Self::new();
        curve.push_rect(rect);
        curve
    }

    /// Get the number of verbs in the curve.
    pub fn len(&self) -> usize {
        self.verbs.len()
    }

    /// Check if the curve is empty.
    pub fn is_empty(&self) -> bool {
        self.verbs.is_empty()
    }

    /// Check if the curve is valid.
    pub fn is_valid(&self) -> bool {
        self.verbs.len() > 1
    }

    /// Check if the curve is closed.
    pub fn is_closed(&self) -> bool {
        self.verbs.last() == Some(&CurveVerb::Close)
    }

    /// Get the bounds of the curve.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    /// Clear the curve, retaining the allocated memory for reuse.
    pub fn clear(&mut self) {
        self.verbs.clear();
        self.points.clear();
        self.bounds = Rect::ZERO;
    }

    fn push_point(&mut self, point: Point) {
        self.points.push(point);
        self.bounds = self.bounds.include(point);
    }

    fn push_verb(&mut self, verb: CurveVerb) {
        self.verbs.push(verb);
    }

    /// Move to a `point`.
    pub fn move_to(&mut self, point: Point) {
        self.push_verb(CurveVerb::Move);
        self.push_point(point);
    }

    /// Draw a line to a `point`.
    pub fn line_to(&mut self, point: Point) {
        self.push_verb(CurveVerb::Line);
        self.push_point(point);
    }

    /// Draw a quadratic bezier curve to a `point`, with a control point `control`.
    pub fn quad_to(&mut self, control: Point, point: Point) {
        self.push_verb(CurveVerb::Quad);
        self.push_point(control);
        self.push_point(point);
    }

    /// Draw a cubic bezier curve to a `point`, with control points `a` and `b`.
    pub fn cubic_to(&mut self, a: Point, b: Point, point: Point) {
        self.push_verb(CurveVerb::Cubic);
        self.push_point(a);
        self.push_point(b);
        self.push_point(point);
    }

    /// Close the contour.
    pub fn close(&mut self) {
        if !self.is_closed() && !self.is_empty() {
            self.verbs.push(CurveVerb::Close);
        }
    }

    /// Push a rectangle to the curve.
    pub fn push_rect(&mut self, rect: Rect) {
        self.move_to(rect.top_left());
        self.line_to(rect.top_right());
        self.line_to(rect.bottom_right());
        self.line_to(rect.bottom_left());
        self.close();
    }

    /// Push a rectangle with rounded corners to the curve.
    pub fn push_rect_with_radius(&mut self, rect: Rect, radius: BorderRadius) {
        self.move_to(rect.top_left() + Vector::new(radius.top_left, 0.0));

        self.line_to(rect.top_right() + Vector::new(-radius.top_right, 0.0));
        self.quad_to(
            rect.top_right(),
            rect.top_right() + Vector::new(0.0, radius.top_right),
        );

        self.line_to(rect.bottom_right() + Vector::new(0.0, -radius.bottom_right));
        self.quad_to(
            rect.bottom_right(),
            rect.bottom_right() + Vector::new(-radius.bottom_right, 0.0),
        );

        self.line_to(rect.bottom_left() + Vector::new(radius.bottom_left, 0.0));
        self.quad_to(
            rect.bottom_left(),
            rect.bottom_left() + Vector::new(0.0, -radius.bottom_left),
        );

        self.line_to(rect.top_left() + Vector::new(0.0, radius.top_left));
        self.quad_to(
            rect.top_left(),
            rect.top_left() + Vector::new(radius.top_left, 0.0),
        );

        self.close();
    }

    /// Transform the curve by the given affine transform.
    pub fn transform(&mut self, transform: Affine) {
        for point in &mut self.points {
            *point = transform * *point;
        }

        self.bounds = self.bounds.transform(transform);
    }

    /// Get an iterator over the curve segments.
    pub fn iter(&self) -> CurveIter {
        CurveIter {
            curve: self,
            verb: 0,
            point: 0,
        }
    }

    fn quadratic_roots(a: f32, b: f32, c: f32) -> [f32; 2] {
        if a.abs() < 1e-6 {
            return [-c / b, f32::NAN];
        }

        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return [f32::NAN, f32::NAN];
        }

        let sqrt = discriminant.sqrt();

        let x1 = (-b + sqrt) / (2.0 * a);
        let x2 = (-b - sqrt) / (2.0 * a);

        [x1, x2]
    }

    fn cubic_roots(a: f32, b: f32, c: f32, d: f32) -> [f32; 3] {
        if a.abs() < 1e-6 {
            let [x1, x2] = Self::quadratic_roots(b, c, d);
            return [x1, x2, f32::NAN];
        }

        let b = b / a;
        let c = c / a;
        let d = d / a;

        let q = (3.0 * c - b * b) / 9.0;
        let r = (9.0 * b * c - 27.0 * d - 2.0 * b * b * b) / 54.0;

        let discriminant = q * q * q + r * r;

        if discriminant >= 0.0 {
            let s = r + discriminant.sqrt();
            let t = r - discriminant.sqrt();

            let s = s.abs().powf(1.0 / 3.0) * s.signum();
            let t = t.abs().powf(1.0 / 3.0) * t.signum();

            let x1 = -b / 3.0 + (s + t);
            let x2 = -b / 3.0 - (s + t) / 2.0;
            let x3 = -b / 3.0 - (s + t) / 2.0;

            [x1, x2, x3]
        } else {
            let theta = (-r / (q * q * q)).acos().abs();
            let sqrt_q = q.abs().sqrt();

            let x1 = -2.0 * sqrt_q * (theta / 3.0).cos() - b / 3.0;
            let x2 = 2.0 * sqrt_q * ((theta + 2.0 * PI) / 3.0).cos() - b / 3.0;
            let x3 = 2.0 * sqrt_q * ((theta + 4.0 * PI) / 3.0).cos() - b / 3.0;

            [x1, x2, x3]
        }
    }

    fn lerps(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    fn quadratic_bezier(s: f32, c0: f32, e: f32, t: f32) -> f32 {
        Self::lerps(Self::lerps(s, c0, t), Self::lerps(c0, e, t), t)
    }

    fn cubic_bezier(s: f32, c0: f32, c1: f32, e: f32, t: f32) -> f32 {
        Self::lerps(
            Self::quadratic_bezier(s, c0, c1, t),
            Self::quadratic_bezier(c0, c1, e, t),
            t,
        )
    }

    /// Check if the curve contains a `point` using the given `rule`.
    pub fn contains(&self, point: Point, rule: FillRule) -> bool {
        if !self.bounds.contains(point) || self.is_empty() {
            return false;
        }

        match rule {
            FillRule::NonZero => self.contains_even_odd(point),
            FillRule::EvenOdd => self.contains_even_odd(point),
        }
    }

    // check if the curve contains a point using the even-odd rule
    //
    // this works by counting the number of times a ray starting at the `point`
    // and extending to the right along the x-axis intersects the curve
    fn contains_even_odd(&self, p: Point) -> bool {
        let mut crossings = 0;
        let mut s = *self.points.last().unwrap();

        for segment in self.iter() {
            match segment {
                CurveSegment::Move(e) => {
                    s = e;
                }
                CurveSegment::Line(e) => {
                    crossings += Self::line_intersections(s, e, p);
                    s = e;
                }
                CurveSegment::Quad(c0, e) => {
                    crossings += Self::quad_intersections(s, c0, e, p);
                }
                CurveSegment::Cubic(c0, c1, e) => {
                    crossings += Self::cubic_intersections(s, c0, c1, e, p);
                }
                CurveSegment::Close => {}
            }
        }

        crossings % 2 == 1
    }

    fn line_intersections(s: Point, e: Point, p: Point) -> usize {
        let a = e.y - s.y;
        let b = s.y - p.y;

        let t = -b / a;

        let is_on_curve = (0.0..1.0).contains(&t);
        let is_right = Self::lerps(s.x, e.x, t) >= p.x;

        (is_on_curve && is_right) as usize
    }

    fn quad_intersections(s: Point, c0: Point, e: Point, p: Point) -> usize {
        let a = s.y - 2.0 * c0.y + e.y;
        let b = 2.0 * (c0.y - s.y);
        let c = s.y - p.y;

        let is_valid = |t: f32| {
            let is_on_curve = (0.0..1.0).contains(&t);
            let is_right = Self::quadratic_bezier(s.x, c0.x, e.x, t) >= p.x;
            is_on_curve && is_right
        };

        let [t1, t2] = Self::quadratic_roots(a, b, c);

        is_valid(t1) as usize + is_valid(t2) as usize
    }

    fn cubic_intersections(s: Point, c0: Point, c1: Point, e: Point, p: Point) -> usize {
        let a = -s.y + 3.0 * c0.y - 3.0 * c1.y + e.y;
        let b = 3.0 * (s.y - 2.0 * c0.y + c1.y);
        let c = 3.0 * (c0.y - s.y);
        let d = s.y - p.y;

        let is_valid = |t: f32| {
            let is_on_curve = (0.0..1.0).contains(&t);
            let is_right = Self::cubic_bezier(s.x, c0.x, c1.x, e.x, t) >= p.x;
            is_on_curve && is_right
        };

        let [t1, t2, t3] = Self::cubic_roots(a, b, c, d);

        is_valid(t1) as usize + is_valid(t2) as usize + is_valid(t3) as usize
    }
}

impl<'a> IntoIterator for &'a Curve {
    type Item = CurveSegment;
    type IntoIter = CurveIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl From<Rect> for Curve {
    fn from(rect: Rect) -> Self {
        Self::rect(rect)
    }
}

/// An iterator over the segments of a curve.
pub struct CurveIter<'a> {
    curve: &'a Curve,
    verb: usize,
    point: usize,
}

impl<'a> Iterator for CurveIter<'a> {
    type Item = CurveSegment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.verb >= self.curve.verbs.len() {
            return None;
        }

        let verb = self.curve.verbs[self.verb];
        self.verb += 1;

        let segment = match verb {
            CurveVerb::Move => CurveSegment::Move(
                // move
                self.curve.points[self.point],
            ),
            CurveVerb::Line => CurveSegment::Line(
                // line
                self.curve.points[self.point],
            ),
            CurveVerb::Quad => CurveSegment::Quad(
                // quadratic
                self.curve.points[self.point],
                self.curve.points[self.point + 1],
            ),
            CurveVerb::Cubic => CurveSegment::Cubic(
                // cubic
                self.curve.points[self.point],
                self.curve.points[self.point + 1],
                self.curve.points[self.point + 2],
            ),
            CurveVerb::Close => CurveSegment::Close,
        };

        self.point += verb.points();

        Some(segment)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.curve.verbs.len() - self.verb;
        (remaining, Some(remaining))
    }
}
