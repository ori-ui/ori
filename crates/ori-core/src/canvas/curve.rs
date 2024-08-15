use std::f32::consts::{PI, SQRT_2};

use crate::layout::{Affine, Point, Rect, Size, Vector};

use super::{BorderRadius, BorderWidth, FillRule, Stroke};

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

/// A bezier curve.
#[derive(Clone, Debug, PartialEq, Hash)]
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

    /// Create a curve from an oval.
    pub fn oval(oval: Rect) -> Self {
        let mut curve = Self::new();
        curve.push_oval(oval);
        curve
    }

    /// Create a curve from a cicrle.
    pub fn circle(center: Point, radius: f32) -> Self {
        Self::oval(Rect::center_size(center, Size::all(radius * 2.0)))
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

    /// Get the points in the curve.
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Get the verbs in the curve.
    pub fn verbs(&self) -> &[CurveVerb] {
        &self.verbs
    }

    /// Get the last point in the curve.
    pub fn last_point(&self) -> Option<Point> {
        self.points.last().copied()
    }

    fn push_point(&mut self, point: Point) {
        if self.points.is_empty() {
            self.bounds = Rect::new(point, point);
        } else {
            self.bounds = self.bounds.include(point);
        }

        self.points.push(point);
    }

    fn push_verb(&mut self, verb: CurveVerb) {
        self.verbs.push(verb);
    }

    /// Move to a `point`.
    #[track_caller]
    pub fn move_to(&mut self, point: Point) {
        debug_assert!(!point.is_nan());

        self.push_verb(CurveVerb::Move);
        self.push_point(point);
    }

    /// Draw a line to a `point`.
    #[track_caller]
    pub fn line_to(&mut self, point: Point) {
        debug_assert!(!point.is_nan());

        self.push_verb(CurveVerb::Line);
        self.push_point(point);
    }

    /// Draw a quadratic bezier curve to a `point`, with a control point `control`.
    #[track_caller]
    pub fn quad_to(&mut self, control: Point, point: Point) {
        debug_assert!(!control.is_nan());
        debug_assert!(!point.is_nan());

        self.push_verb(CurveVerb::Quad);
        self.push_point(control);
        self.push_point(point);
    }

    /// Draw a cubic bezier curve to a `point`, with control points `a` and `b`.
    #[track_caller]
    pub fn cubic_to(&mut self, a: Point, b: Point, point: Point) {
        debug_assert!(!a.is_nan());
        debug_assert!(!b.is_nan());

        self.push_verb(CurveVerb::Cubic);
        self.push_point(a);
        self.push_point(b);
        self.push_point(point);
    }

    /// Draw a conic curve to a `point`, with a control point `control` and a `weight`.
    ///
    /// Conic curves are approximated by quadratic bezier curves.
    pub fn conic_to(&mut self, control: Point, point: Point, weight: f32) {
        let last_point = self.points.last().copied().unwrap_or(Point::ZERO);

        let conic = Conic {
            start: last_point,
            control,
            end: point,
            weight,
        };

        if let Some(pow2) = conic.compute_quad_pow2(0.25) {
            let mut points = [Point::ZERO; 64];
            let len = conic.chop_into_quads_pow2(pow2, &mut points);

            let mut offset = 1;
            for _ in 0..len {
                let control = points[offset];
                let end = points[offset + 1];
                self.quad_to(control, end);
                offset += 2;
            }
        }
    }

    /// Close the contour.
    pub fn close(&mut self) {
        if !self.is_closed() && !self.is_empty() {
            self.verbs.push(CurveVerb::Close);
        }
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

    /// Stroke the `curve` with the given `stroke`.
    pub fn stroke_curve(&mut self, curve: &Curve, stroke: Stroke) {
        self.stroke_impl(curve, stroke);
    }

    pub(crate) fn append_reverse(&mut self, curve: &Curve) {
        let mut offset = curve.points.len() - 1;
        for verb in curve.verbs.iter().rev() {
            match verb {
                CurveVerb::Move => break,
                CurveVerb::Line => {
                    let p = curve.points[offset - 1];
                    offset -= 1;

                    self.line_to(p);
                }
                CurveVerb::Quad => {
                    let c = curve.points[offset - 1];
                    let p = curve.points[offset - 2];
                    offset -= 2;

                    self.quad_to(c, p);
                }
                CurveVerb::Cubic => {
                    let c1 = curve.points[offset - 1];
                    let c2 = curve.points[offset - 2];
                    let p = curve.points[offset - 3];
                    offset -= 3;

                    self.cubic_to(c1, c2, p);
                }
                CurveVerb::Close => todo!(),
            }
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

    /// Push an oval to the curve.
    pub fn push_oval(&mut self, oval: Rect) {
        let weight = SQRT_2 / 2.0;

        self.move_to(oval.top());
        self.conic_to(oval.top_left(), oval.left(), weight);
        self.conic_to(oval.bottom_left(), oval.bottom(), weight);
        self.conic_to(oval.bottom_right(), oval.right(), weight);
        self.conic_to(oval.top_right(), oval.top(), weight);
        self.close();
    }

    /// Push a rectangle with rounded corners to the curve.
    pub fn push_rect_with_radius(&mut self, rect: Rect, radius: BorderRadius) {
        self.move_to(rect.top_left() + Vector::new(radius.top_left, 0.0));

        self.line_to(rect.top_right() + Vector::new(-radius.top_right, 0.0));
        self.cubic_to(
            rect.top_right() + Vector::new(-radius.top_right * 0.45, 0.0),
            rect.top_right() + Vector::new(0.0, radius.top_right * 0.45),
            rect.top_right() + Vector::new(0.0, radius.top_right),
        );

        self.line_to(rect.bottom_right() + Vector::new(0.0, -radius.bottom_right));
        self.cubic_to(
            rect.bottom_right() + Vector::new(0.0, -radius.bottom_right * 0.45),
            rect.bottom_right() + Vector::new(-radius.bottom_right * 0.45, 0.0),
            rect.bottom_right() + Vector::new(-radius.bottom_right, 0.0),
        );

        self.line_to(rect.bottom_left() + Vector::new(radius.bottom_left, 0.0));
        self.cubic_to(
            rect.bottom_left() + Vector::new(radius.bottom_left * 0.45, 0.0),
            rect.bottom_left() + Vector::new(0.0, -radius.bottom_left * 0.45),
            rect.bottom_left() + Vector::new(0.0, -radius.bottom_left),
        );

        self.line_to(rect.top_left() + Vector::new(0.0, radius.top_left));
        self.cubic_to(
            rect.top_left() + Vector::new(0.0, radius.top_left * 0.45),
            rect.top_left() + Vector::new(radius.top_left * 0.45, 0.0),
            rect.top_left() + Vector::new(radius.top_left, 0.0),
        );

        self.close();
    }

    fn border_data(radius: BorderRadius, width: BorderWidth, index: usize) -> [f32; 3] {
        let radius: [f32; 4] = radius.into();
        let width: [f32; 4] = width.into();

        let radius = radius[index];
        let start = width[(index + 3) % 4];
        let end = width[index];

        [radius, start, end]
    }

    /// Push the border of a rectangle with rounded corners to the curve.
    pub fn push_rect_with_borders(&mut self, rect: Rect, radius: BorderRadius, width: BorderWidth) {
        let tl = rect.top_left();
        let tr = rect.top_right();
        let br = rect.bottom_right();
        let bl = rect.bottom_left();

        let [r, s, e] = Self::border_data(radius, width, 0);

        if s > 0.0 || e > 0.0 {
            self.move_to(tl + Vector::new(0.0, r));
            self.cubic_to(
                tl + Vector::new(0.0, r * 0.45),
                tl + Vector::new(r * 0.45, 0.0),
                tl + Vector::new(r, 0.0),
            );
            self.line_to(tr + Vector::new(-r, 0.0));

            self.line_to(tr + Vector::new(-r, e));
            self.line_to(tl + Vector::new(r, e));
            self.cubic_to(
                tl + Vector::new((r + s) * 0.45, e),
                tl + Vector::new(s, (r + e) * 0.45),
                tl + Vector::new(s, r),
            );

            self.close();
        }

        let [r, s, e] = Self::border_data(radius, width, 1);

        if s > 0.0 || e > 0.0 {
            self.move_to(tr + Vector::new(-r, 0.0));
            self.cubic_to(
                tr + Vector::new(-r * 0.45, 0.0),
                tr + Vector::new(0.0, r * 0.45),
                tr + Vector::new(0.0, r),
            );
            self.line_to(br + Vector::new(0.0, -r));

            self.line_to(br + Vector::new(-e, -r));
            self.line_to(tr + Vector::new(-e, r));
            self.cubic_to(
                tr + Vector::new(-e, (r + s) * 0.45),
                tr + Vector::new(-(r + e) * 0.45, s),
                tr + Vector::new(-r, s),
            );

            self.close();
        }

        let [r, s, e] = Self::border_data(radius, width, 2);

        if s > 0.0 || e > 0.0 {
            self.move_to(br + Vector::new(0.0, -r));
            self.cubic_to(
                br + Vector::new(0.0, -r * 0.45),
                br + Vector::new(-r * 0.45, 0.0),
                br + Vector::new(-r, 0.0),
            );
            self.line_to(bl + Vector::new(r, 0.0));

            self.line_to(bl + Vector::new(r, -e));
            self.line_to(br + Vector::new(-r, -e));
            self.cubic_to(
                br + Vector::new(-(r + s) * 0.45, -e),
                br + Vector::new(-s, -(r + e) * 0.45),
                br + Vector::new(-s, -r),
            );

            self.close();
        }

        let [r, s, e] = Self::border_data(radius, width, 3);

        if s > 0.0 || e > 0.0 {
            self.move_to(bl + Vector::new(r, 0.0));
            self.cubic_to(
                bl + Vector::new(r * 0.45, 0.0),
                bl + Vector::new(0.0, -r * 0.45),
                bl + Vector::new(0.0, -r),
            );
            self.line_to(tl + Vector::new(0.0, r));

            self.line_to(tl + Vector::new(e, r));
            self.line_to(bl + Vector::new(e, -r));
            self.cubic_to(
                bl + Vector::new(e, -(r + s) * 0.45),
                bl + Vector::new((r + e) * 0.45, -s),
                bl + Vector::new(r, -s),
            );

            self.close();
        }
    }

    pub(crate) fn square_roots(a: f32, b: f32, c: f32) -> [f32; 2] {
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

    pub(crate) fn cube_roots(a: f32, b: f32, c: f32, d: f32) -> [f32; 3] {
        if a.abs() < 1e-6 {
            let [x1, x2] = Self::square_roots(b, c, d);
            return [x1, x2, f32::NAN];
        }

        let a_inv = 1.0 / a;
        let a = b * a_inv;
        let b = c * a_inv;
        let c = d * a_inv;

        let q = (a * a - b * 3.0) / 9.0;
        let r = (2.0 * a * a * a - 9.0 * a * b + 27.0 * c) / 54.0;

        let q3 = q * q * q;
        let r2_minus_q3 = r * r - q3;
        let adiv3 = a / 3.0;

        if r2_minus_q3 < 0.0 {
            // we have 3 real roots
            let theta = (r / q3.sqrt()).acos();
            let neg2_root_q = -2.0 * q.sqrt();

            let x1 = neg2_root_q * (theta / 3.0).cos() - adiv3;
            let x2 = neg2_root_q * ((theta + 2.0 * PI) / 3.0).cos() - adiv3;
            let x3 = neg2_root_q * ((theta - 2.0 * PI) / 3.0).cos() - adiv3;

            [x1, x2, x3]
        } else {
            // we have 1 real root
            let mut a = (r.abs() + r2_minus_q3.sqrt()).cbrt();

            if r < 0.0 {
                a = -a;
            }

            if a != 0.0 {
                a += q / a;
            }

            [a - adiv3, f32::NAN, f32::NAN]
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
                    crossings += Self::line_intersection_count(s, e, p);
                    s = e;
                }
                CurveSegment::Quad(c0, e) => {
                    crossings += Self::quad_intersection_count(s, c0, e, p);
                    s = e;
                }
                CurveSegment::Cubic(c0, c1, e) => {
                    crossings += Self::cubic_intersection_count(s, c0, c1, e, p);
                    s = e;
                }
                CurveSegment::Close => {}
            }
        }

        crossings % 2 == 1
    }

    fn line_intersection_count(s: Point, e: Point, p: Point) -> usize {
        let a = e.y - s.y;
        let b = s.y - p.y;

        let t = -b / a;

        let is_on_curve = (0.0..=1.0).contains(&t);
        let is_right = Self::lerps(s.x, e.x, t) >= p.x;

        (is_on_curve && is_right) as usize
    }

    pub(crate) fn quad_intersections(s: Point, c0: Point, e: Point, p: Point) -> [f32; 2] {
        let a = s.y - 2.0 * c0.y + e.y;
        let b = 2.0 * (c0.y - s.y);
        let c = s.y - p.y;

        Self::square_roots(a, b, c)
    }

    fn quad_intersection_count(s: Point, c0: Point, e: Point, p: Point) -> usize {
        let [t1, t2] = Self::quad_intersections(s, c0, e, p);

        let is_valid = |t: f32| {
            let is_on_curve = (0.0..=1.0).contains(&t);
            let is_right = Self::quadratic_bezier(s.x, c0.x, e.x, t) >= p.x;

            is_on_curve && is_right
        };

        is_valid(t1) as usize + is_valid(t2) as usize
    }

    pub(crate) fn cubic_intersections(
        s: Point,
        c0: Point,
        c1: Point,
        e: Point,
        p: Point,
    ) -> [f32; 3] {
        let a = -s.y + 3.0 * c0.y - 3.0 * c1.y + e.y;
        let b = 3.0 * (s.y - 2.0 * c0.y + c1.y);
        let c = 3.0 * (c0.y - s.y);
        let d = s.y - p.y;

        Self::cube_roots(a, b, c, d)
    }

    fn cubic_intersection_count(s: Point, c0: Point, c1: Point, e: Point, p: Point) -> usize {
        let is_valid = |t: f32| {
            let is_on_curve = (0.0..=1.0).contains(&t);
            let is_right = Self::cubic_bezier(s.x, c0.x, c1.x, e.x, t) >= p.x;
            is_on_curve && is_right
        };

        let [t1, t2, t3] = Self::cubic_intersections(s, c0, c1, e, p);

        is_valid(t1) as usize + is_valid(t2) as usize + is_valid(t3) as usize
    }
}

fn between(a: f32, b: f32, c: f32) -> bool {
    (a - b) * (c - b) <= 0.0
}

#[derive(Clone, Copy, Debug)]
struct Conic {
    start: Point,
    control: Point,
    end: Point,
    weight: f32,
}

impl Conic {
    fn compute_quad_pow2(&self, tolerance: f32) -> Option<u8> {
        if tolerance < 0.0 || tolerance.is_infinite() {
            return None;
        }

        if self.start.is_infinite() || self.control.is_infinite() || self.end.is_infinite() {
            return None;
        }

        const MAX_CONIC_TO_QUAD_POW2: usize = 4;

        let a = self.weight - 1.0;
        let k = a / (4.0 * (2.0 + a));
        let x = k * (self.start.x - 2.0 * self.control.x + self.end.x);
        let y = k * (self.start.y - 2.0 * self.control.y + self.end.y);

        let mut error = f32::sqrt(x * x + y * y);
        let mut pow2 = 0;

        for _ in 0..MAX_CONIC_TO_QUAD_POW2 {
            if error < tolerance {
                break;
            }

            error *= 0.25;
            pow2 += 1;
        }

        Some(pow2.max(1))
    }

    fn chop_into_quads_pow2(&self, pow2: u8, points: &mut [Point]) -> u8 {
        points[0] = self.start;
        self.subdivide(&mut points[1..], pow2);

        let quad_count = 1 << pow2;
        let point_count = 2 * quad_count + 1;

        if points.iter().take(point_count).any(|n| n.is_infinite()) {
            for p in points.iter_mut().take(point_count - 1).skip(1) {
                *p = self.control;
            }
        }

        quad_count as u8
    }

    fn subdivide<'a>(&self, points: &'a mut [Point], level: u8) -> &'a mut [Point] {
        if level == 0 {
            points[0] = self.control;
            points[1] = self.end;
            &mut points[2..]
        } else {
            let (mut dst_a, mut dst_b) = self.chop();

            let start_y = self.start.y;
            let end_y = self.end.y;

            if between(start_y, self.control.y, end_y) {
                let mid_y = dst_a.end.y;

                if !between(start_y, mid_y, end_y) {
                    let closer = if (mid_y - start_y).abs() < (mid_y - end_y).abs() {
                        start_y
                    } else {
                        end_y
                    };

                    dst_a.end.y = closer;
                    dst_b.start.y = closer;
                }

                if !between(start_y, dst_a.control.y, dst_a.end.y) {
                    dst_a.control.y = start_y;
                }

                if !between(dst_b.start.y, dst_b.control.y, end_y) {
                    dst_b.control.y = end_y;
                }

                debug_assert!(between(start_y, dst_a.control.y, dst_a.end.y));
                debug_assert!(between(dst_a.control.y, dst_a.end.y, dst_b.control.y));
                debug_assert!(between(dst_a.end.y, dst_b.control.y, end_y));
            }

            let points = dst_a.subdivide(points, level - 1);
            dst_b.subdivide(points, level - 1)
        }
    }

    fn chop(&self) -> (Conic, Conic) {
        let scale = Vector::all(1.0 / (1.0 + self.weight));
        let new_weight = f32::sqrt(0.5 + self.weight * 0.5);
        let ww = Vector::all(self.weight);

        let wc = self.control * ww;
        let mut m = (self.start + wc.to_vector() * 2.0 + self.end.to_vector()) * scale * 0.5;

        if m.is_infinite() {
            let wd = self.weight as f64;
            let w2 = wd * 2.0;
            let scale_half = 1.0 / (1.0 + wd) * 0.5;
            m.x = ((self.start.x as f64 + w2 * self.control.x as f64 + self.end.x as f64)
                * scale_half) as f32;

            m.y = ((self.start.y as f64 + w2 * self.control.y as f64 + self.end.y as f64)
                * scale_half) as f32;
        }

        (
            Conic {
                start: self.start,
                control: (self.start + wc.to_vector()) * scale,
                end: m,
                weight: new_weight,
            },
            Conic {
                start: m,
                control: (self.end + wc.to_vector()) * scale,
                end: self.end,
                weight: new_weight,
            },
        )
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
