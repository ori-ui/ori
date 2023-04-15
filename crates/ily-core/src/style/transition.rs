use std::{
    any::Any,
    ops::{Add, Mul},
};

use ily_graphics::Color;
use smallvec::SmallVec;
use smol_str::SmolStr;

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

pub trait Transitionable
where
    Self: Mul<f32, Output = Self> + Add<Output = Self> + PartialEq + Copy,
{
}

impl<T: Mul<f32, Output = T> + Add<Output = T> + PartialEq + Copy> Transitionable for T {}

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

    pub fn update(&mut self, to: T, transition: Option<StyleTransition>, delta: f32) -> (T, bool) {
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

#[derive(Clone, Debug, Default)]
pub struct TransitionStates {
    units: SmallVec<[(SmolStr, TransitionState<f32>); 4]>,
    colors: SmallVec<[(SmolStr, TransitionState<Color>); 4]>,
}

impl TransitionStates {
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

    pub fn transition_unit(
        &mut self,
        name: &str,
        value: f32,
        transition: Option<StyleTransition>,
        delta: f32,
    ) -> (f32, bool) {
        if let Some(state) = self.find_unit(name) {
            return state.update(value, transition, delta);
        }

        let mut state = TransitionState::default();
        let result = state.update(value, transition, delta);

        self.units.push((name.into(), state));

        result
    }

    pub fn transition_color(
        &mut self,
        name: &str,
        value: Color,
        transition: Option<StyleTransition>,
        delta: f32,
    ) -> (Color, bool) {
        if let Some(state) = self.find_color(name) {
            return state.update(value, transition, delta);
        }

        let mut state = TransitionState::default();
        let result = state.update(value, transition, delta);

        self.colors.push((name.into(), state));

        result
    }

    pub fn transition_any<T: Any>(
        &mut self,
        name: &str,
        value: &mut T,
        transition: Option<StyleTransition>,
        delta: f32,
    ) -> bool {
        if let Some(value) = <dyn Any>::downcast_mut::<f32>(value) {
            let (new_value, redraw) = self.transition_unit(name, *value, transition, delta);
            *value = new_value;
            return redraw;
        }

        if let Some(value) = <dyn Any>::downcast_mut::<Color>(value) {
            let (new_value, redraw) = self.transition_color(name, *value, transition, delta);
            *value = new_value;
            return redraw;
        }

        false
    }
}
