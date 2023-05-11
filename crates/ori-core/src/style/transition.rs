use std::{
    any::Any,
    ops::{Add, Mul},
};

use ori_graphics::Color;
use smallvec::SmallVec;
use smol_str::SmolStr;

/// A style transition.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StyleTransition {
    pub duration: f32,
}

impl StyleTransition {
    pub const fn new(duration: f32) -> Self {
        Self { duration }
    }

    pub const fn instant() -> Self {
        Self::new(0.0)
    }
}

impl From<f32> for StyleTransition {
    fn from(duration: f32) -> Self {
        Self::new(duration)
    }
}

/// A value that can be transitioned.
pub trait Transitionable
where
    Self: Mul<f32, Output = Self> + Add<Output = Self> + PartialEq + Copy,
{
}

impl<T: Mul<f32, Output = T> + Add<Output = T> + PartialEq + Copy> Transitionable for T {}

/// The state of a transition.
#[derive(Clone, Copy, Debug)]
pub struct TransitionState<T> {
    pub from: Option<T>,
    pub to: Option<T>,
    pub prev_transition: Option<StyleTransition>,
    pub transition: Option<StyleTransition>,
    pub elapsed: f32,
}

impl<T> Default for TransitionState<T> {
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

impl<T: Transitionable> TransitionState<T> {
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

        // TODO: this is some bs
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

#[derive(Clone, Debug, Default)]
pub struct TransitionStates {
    units: SmallVec<[(SmolStr, TransitionState<f32>); 4]>,
    colors: SmallVec<[(SmolStr, TransitionState<Color>); 4]>,
}

impl TransitionStates {
    /// Creates a new `TransitionStates`.
    pub const fn new() -> Self {
        Self {
            units: SmallVec::new_const(),
            colors: SmallVec::new_const(),
        }
    }

    fn find_unit(&mut self, name: &str) -> Option<&mut TransitionState<f32>> {
        for (key, value) in &mut self.units {
            if key == name {
                return Some(value);
            }
        }

        None
    }

    fn find_color(&mut self, name: &str) -> Option<&mut TransitionState<Color>> {
        for (key, value) in &mut self.colors {
            if key == name {
                return Some(value);
            }
        }

        None
    }

    /// Transitions an `f32` value.
    pub fn transition_f32(
        &mut self,
        name: &str,
        value: f32,
        transition: Option<StyleTransition>,
    ) -> f32 {
        if let Some(state) = self.find_unit(name) {
            return state.get(value, transition);
        }

        let mut state = TransitionState::default();
        let result = state.get(value, transition);

        self.units.push((name.into(), state));

        result
    }

    /// Transitions a [`Color`] value.
    pub fn transition_color(
        &mut self,
        name: &str,
        value: Color,
        transition: Option<StyleTransition>,
    ) -> Color {
        if let Some(state) = self.find_color(name) {
            return state.get(value, transition);
        }

        let mut state = TransitionState::default();
        let result = state.get(value, transition);

        self.colors.push((name.into(), state));

        result
    }

    pub(crate) fn transition_any<T: Any>(
        &mut self,
        name: &str,
        value: &mut T,
        transition: Option<StyleTransition>,
    ) {
        if let Some(value) = <dyn Any>::downcast_mut::<f32>(value) {
            *value = self.transition_f32(name, *value, transition);
        }

        if let Some(value) = <dyn Any>::downcast_mut::<Color>(value) {
            *value = self.transition_color(name, *value, transition);
        }
    }

    /// Updates the transitions.
    pub fn update(&mut self, delta: f32) -> bool {
        let mut redraw = false;

        for (_, state) in &mut self.units {
            redraw |= state.update(delta);
        }

        for (_, state) in &mut self.colors {
            redraw |= state.update(delta);
        }

        redraw
    }
}
