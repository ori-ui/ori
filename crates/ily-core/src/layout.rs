use glam::Vec2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxConstraints {
    pub min: Vec2,
    pub max: Vec2,
}

impl BoxConstraints {
    pub const UNBOUNDED: Self = Self {
        min: Vec2::ZERO,
        max: Vec2::splat(f32::INFINITY),
    };

    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self {
            min: min.ceil(),
            max: max.ceil(),
        }
    }

    pub fn window(width: u32, height: u32) -> Self {
        Self {
            min: Vec2::ZERO,
            max: Vec2::new(width as f32, height as f32),
        }
    }

    pub fn loose(self) -> Self {
        Self {
            min: Vec2::ZERO,
            max: self.max,
        }
    }
}
