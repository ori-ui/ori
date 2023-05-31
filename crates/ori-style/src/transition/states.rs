use std::any::Any;

use ori_graphics::Color;
use smallvec::SmallVec;

use crate::{StyleAttributeKey, StyleTransition, StyleTransitionState};

/// A collection of transition states.
#[derive(Clone, Debug, Default)]
pub struct StyleTransitionStates {
    lengths: SmallVec<[(StyleAttributeKey, StyleTransitionState<f32>); 4]>,
    colors: SmallVec<[(StyleAttributeKey, StyleTransitionState<Color>); 4]>,
}

impl StyleTransitionStates {
    /// Creates a new `TransitionStates`.
    pub const fn new() -> Self {
        Self {
            lengths: SmallVec::new_const(),
            colors: SmallVec::new_const(),
        }
    }

    fn find_length(&mut self, name: &str) -> Option<&mut StyleTransitionState<f32>> {
        for (key, value) in &mut self.lengths {
            if key == name {
                return Some(value);
            }
        }

        None
    }

    fn find_color(&mut self, name: &str) -> Option<&mut StyleTransitionState<Color>> {
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
        if let Some(state) = self.find_length(name) {
            return state.get(value, transition);
        }

        let mut state = StyleTransitionState::default();
        let result = state.get(value, transition);

        self.lengths.push((name.into(), state));

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

        let mut state = StyleTransitionState::default();
        let result = state.get(value, transition);

        self.colors.push((name.into(), state));

        result
    }

    pub(crate) fn transition_any_inner<T: Any>(
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

    /// Transitions a value if it is an `f32` or a [`Color`], otherwise does nothing.
    pub fn transition_any<T: 'static>(
        &mut self,
        name: &str,
        mut value: T,
        transition: Option<StyleTransition>,
    ) -> T {
        self.transition_any_inner(name, &mut value, transition);
        value
    }

    /// Updates the transitions.
    pub fn update(&mut self, delta: f32) -> bool {
        let mut redraw = false;

        for (_, state) in &mut self.lengths {
            redraw |= state.update(delta);
        }

        for (_, state) in &mut self.colors {
            redraw |= state.update(delta);
        }

        redraw
    }
}
