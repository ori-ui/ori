use crate::{StyleTransition, Transitionable};

/// The state of a transition.
#[derive(Clone, Copy, Debug)]
pub struct StyleTransitionState<T> {
    pub from: Option<T>,
    pub to: Option<T>,
    pub prev_transition: Option<StyleTransition>,
    pub transition: Option<StyleTransition>,
    pub elapsed: f32,
}

impl<T> Default for StyleTransitionState<T> {
    fn default() -> Self {
        Self {
            from: None,
            to: None,
            prev_transition: None,
            transition: None,
            elapsed: 0.0,
        }
    }
}

impl<T: Transitionable> StyleTransitionState<T> {
    fn mix(from: T, to: T, progress: f32) -> T {
        from * (1.0 - progress) + to * progress
    }

    fn transition(&self) -> Option<StyleTransition> {
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

    /// Gets the current value of the transition.
    ///
    /// # Arguments
    /// - `to`: The value to transition to.
    /// - `transition`: The transition to use.
    pub fn get(&mut self, to: T, transition: Option<StyleTransition>) -> T {
        if self.from.is_none() {
            self.from = Some(to);
        }

        if self.transition != transition || self.to != Some(to) {
            if let (Some(prev), Some(new)) = (self.transition, transition) {
                let progress = self.elapsed / prev.duration;

                self.elapsed = new.duration - (new.duration * progress);
            } else {
                self.elapsed = 0.0;
            }

            self.prev_transition = self.transition;
            self.transition = transition;
            self.to = Some(to);
        }

        if self.is_complete() {
            return to;
        }

        self.from.unwrap()
    }

    /// Updates the transition.
    pub fn update(&mut self, delta: f32) -> bool {
        if self.is_complete() {
            return false;
        }

        let Some(transition) = self.transition() else {
            return false;
        };

        let Some(to) = self.to else {
            return false;
        };

        // FIXME: this is some bs
        if self.elapsed == 0.0 {
            self.elapsed = f32::EPSILON;
            return true;
        }

        let remaining = transition.duration - self.elapsed;
        let delta = delta.min(remaining);
        let progress = delta / remaining;

        let value = Self::mix(self.from.unwrap(), to, progress);
        self.from = Some(value);

        self.elapsed += delta;

        true
    }
}
