use std::ops::{Add, Mul};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Transition {
    pub duration: f32,
}

impl Transition {
    pub const fn new(duration: f32) -> Self {
        Self { duration }
    }

    pub const fn instant() -> Self {
        Self::new(0.0)
    }
}

#[derive(Clone, Debug, Default)]
pub struct TransitionState<T> {
    pub from: Option<T>,
    pub to: Option<T>,
    pub prev_transition: Option<Transition>,
    pub transition: Option<Transition>,
    pub elapsed: f32,
}

impl<T> TransitionState<T>
where
    T: Mul<f32, Output = T> + Add<Output = T> + PartialEq + Copy + Default,
{
    fn mix(from: T, to: T, progress: f32) -> T {
        from * (1.0 - progress) + to * progress
    }

    fn transition(&self) -> Option<Transition> {
        if let Some(transition) = self.transition {
            return Some(transition);
        }

        self.prev_transition
    }

    fn is_complete(&self) -> bool {
        if let Some(transition) = self.transition() {
            return self.elapsed >= transition.duration;
        }

        true
    }

    pub fn update(&mut self, to: T, transition: Option<Transition>, delta: f32) -> (T, bool) {
        if self.from.is_none() {
            self.from = Some(to);
        }

        if self.transition != transition || self.to != Some(to) {
            self.prev_transition = self.transition;
            self.transition = transition;
            self.to = Some(to);

            self.elapsed = 0.0;

            return (self.from.unwrap(), true);
        }

        if self.is_complete() {
            return (to, false);
        }

        let Some(transition) = self.transition() else {
            return (to, false);
        };

        let remaining = transition.duration - self.elapsed;
        let delta = delta.min(remaining);
        let progress = delta / remaining;

        let value = Self::mix(self.from.unwrap(), to, progress);
        self.from = Some(value);

        self.elapsed += delta;

        (value, true)
    }
}
