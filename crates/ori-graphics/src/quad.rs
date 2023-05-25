use std::f32::consts::FRAC_PI_2;

use glam::Vec2;

use crate::{Color, Curve, Rect};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    pub rect: Rect,
    pub background: Color,
    pub border_radius: [f32; 4],
    pub border_width: f32,
    pub border_color: Color,
}

impl Default for Quad {
    fn default() -> Self {
        Self {
            rect: Rect::default(),
            background: Color::WHITE,
            border_radius: [0.0; 4],
            border_width: 0.0,
            border_color: Color::BLACK,
        }
    }
}

impl Quad {
    pub fn rounded(self) -> Self {
        Self {
            rect: self.rect.round(),
            ..self
        }
    }

    pub fn inside_curve(self) -> Curve {
        fn corner(curve: &mut Curve, center: Vec2, radius: f32, start_angle: f32) {
            let start = start_angle;
            let end = start_angle + FRAC_PI_2;

            let corner = Curve::arc_center_angle(center, radius, start, end);
            curve.extend(corner);
        }

        let mut curve = Curve::new();

        let [mut tl, mut tr, mut br, mut bl] = self.border_radius;
        tl -= self.border_width;
        tr -= self.border_width;
        br -= self.border_width;
        bl -= self.border_width;

        let rect = self.rect.shrink(self.border_width);

        let ctl = Vec2::new(rect.left() + tl, rect.top() + tl);
        let ctr = Vec2::new(rect.right() - tr, rect.top() + tr);
        let cbr = Vec2::new(rect.right() - br, rect.bottom() - br);
        let cbl = Vec2::new(rect.left() + bl, rect.bottom() - bl);

        corner(&mut curve, ctr, tr, FRAC_PI_2 * 3.0);
        corner(&mut curve, cbr, br, FRAC_PI_2 * 0.0);
        corner(&mut curve, cbl, bl, FRAC_PI_2 * 1.0);
        corner(&mut curve, ctl, tl, FRAC_PI_2 * 2.0);

        curve
    }

    pub fn border_curve(self) -> Curve {
        fn corner(curve: &mut Curve, center: Vec2, radius: f32, start_angle: f32) {
            let start = start_angle;
            let end = start_angle + FRAC_PI_2;

            let corner = Curve::arc_center_angle(center, radius, start, end);
            curve.extend(corner);
        }

        let mut curve = Curve::new();

        let [mut tl, mut tr, mut br, mut bl] = self.border_radius;
        tl -= self.border_width / 2.0;
        tr -= self.border_width / 2.0;
        br -= self.border_width / 2.0;
        bl -= self.border_width / 2.0;

        let rect = self.rect.shrink(self.border_width / 2.0);

        let ctl = Vec2::new(rect.left() + tl, rect.top() + tl);
        let ctr = Vec2::new(rect.right() - tr, rect.top() + tr);
        let cbr = Vec2::new(rect.right() - br, rect.bottom() - br);
        let cbl = Vec2::new(rect.left() + bl, rect.bottom() - bl);

        corner(&mut curve, ctr, tr, FRAC_PI_2 * 3.0);
        corner(&mut curve, cbr, br, FRAC_PI_2 * 0.0);
        corner(&mut curve, cbl, bl, FRAC_PI_2 * 1.0);
        corner(&mut curve, ctl, tl, FRAC_PI_2 * 2.0);

        curve.add_point(curve[0]);

        curve
    }
}
