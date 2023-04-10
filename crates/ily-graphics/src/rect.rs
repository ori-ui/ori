use glam::Vec2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub const ZERO: Self = Self::new(Vec2::ZERO, Vec2::ZERO);

    pub const fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn min_size(min: Vec2, size: Vec2) -> Self {
        Self {
            min,
            max: min + size,
        }
    }

    pub fn size(self) -> Vec2 {
        self.max - self.min
    }

    pub fn center(self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    pub fn contains(self, point: Vec2) -> bool {
        let inside_x = point.x >= self.min.x && point.x <= self.max.x;
        let inside_y = point.y >= self.min.y && point.y <= self.max.y;
        inside_x && inside_y
    }

    pub fn top_left(self) -> Vec2 {
        self.min
    }

    pub fn top_right(self) -> Vec2 {
        Vec2::new(self.max.x, self.min.y)
    }

    pub fn bottom_left(self) -> Vec2 {
        Vec2::new(self.min.x, self.max.y)
    }

    pub fn bottom_right(self) -> Vec2 {
        self.max
    }
}
