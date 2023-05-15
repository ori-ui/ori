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

    pub fn center_size(center: Vec2, size: Vec2) -> Self {
        let half_size = size / 2.0;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    pub fn round(self) -> Self {
        Self {
            min: self.min.round(),
            max: self.max.round(),
        }
    }

    pub fn ceil(self) -> Self {
        Self {
            min: self.min.floor(),
            max: self.max.ceil(),
        }
    }

    pub fn floor(self) -> Self {
        Self {
            min: self.min.ceil(),
            max: self.max.floor(),
        }
    }

    pub fn shrink(self, amount: f32) -> Self {
        Self {
            min: self.min + Vec2::splat(amount),
            max: self.max - Vec2::splat(amount),
        }
    }

    pub fn size(self) -> Vec2 {
        self.max - self.min
    }

    pub fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn center(self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    pub fn contains(self, point: Vec2) -> bool {
        let inside_x = point.x >= self.min.x && point.x <= self.max.x;
        let inside_y = point.y >= self.min.y && point.y <= self.max.y;
        inside_x && inside_y
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn intersect(self, other: Self) -> Self {
        if self.min.x > other.max.x
            || self.max.x < other.min.x
            || self.min.y > other.max.y
            || self.max.y < other.min.y
        {
            return Self::ZERO;
        }

        Self {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        }
    }

    pub fn left(self) -> f32 {
        self.min.x
    }

    pub fn right(self) -> f32 {
        self.max.x
    }

    pub fn top(self) -> f32 {
        self.min.y
    }

    pub fn bottom(self) -> f32 {
        self.max.y
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

    pub fn right_center(self) -> Vec2 {
        Vec2::new(self.max.x, self.center().y)
    }

    pub fn left_center(self) -> Vec2 {
        Vec2::new(self.min.x, self.center().y)
    }

    pub fn top_center(self) -> Vec2 {
        Vec2::new(self.center().x, self.min.y)
    }

    pub fn bottom_center(self) -> Vec2 {
        Vec2::new(self.center().x, self.max.y)
    }

    pub fn translate(self, offset: impl Into<Vec2>) -> Self {
        let offset = offset.into();

        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }

    pub fn pad(self, padding: impl Into<Vec2>) -> Self {
        let padding = padding.into();

        Self {
            min: self.min - padding,
            max: self.max + padding,
        }
    }
}
