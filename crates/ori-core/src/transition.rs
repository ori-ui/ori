/// Create a linear transition with the given `duration`.
pub fn linear(duration: f32) -> Transition {
    Transition::linear(duration)
}

/// Create an ease transition with the given `duration`.
pub fn ease(duration: f32) -> Transition {
    Transition::ease(duration)
}

/// A transition curve.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum TransitionCurve {
    /// A linear transition curve.
    #[default]
    Linear,
    /// An ease transition curve.
    Ease,
}

impl TransitionCurve {
    /// Evaluate the transition curve at `t`.
    pub fn eval(self, t: f32) -> f32 {
        match self {
            TransitionCurve::Linear => t,
            TransitionCurve::Ease => t * t * (3.0 - 2.0 * t),
        }
    }
}

/// A transition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transition {
    /// The duration of the transition.
    pub duration: f32,
    /// The transition curve.
    pub curve: TransitionCurve,
}

impl Default for Transition {
    fn default() -> Self {
        Self::ease(0.2)
    }
}

impl Transition {
    /// Create a linear transition with the given `duration`.
    pub fn linear(duration: f32) -> Self {
        Self {
            duration,
            curve: TransitionCurve::Linear,
        }
    }

    /// Create an ease transition with the given `duration`.
    pub fn ease(duration: f32) -> Self {
        Self {
            duration,
            curve: TransitionCurve::Ease,
        }
    }

    /// Step the transition.
    pub fn step(&self, t: &mut f32, on: bool, dt: f32) -> bool {
        let sign = if on { 1.0 } else { -1.0 };
        let step = sign * dt / self.duration;
        let from = if on { 0.0 } else { 1.0 };
        let to = if on { 1.0 } else { 0.0 };

        if *t == from {
            *t += sign * 0.0001;
            return true;
        }

        if *t == to {
            return false;
        }

        *t += step;
        *t = t.clamp(0.0, 1.0);

        true
    }

    /// Evaluate the transition curve at `t`.
    ///
    /// The returned value is how _on_ the transition is at `t`.
    /// This is a range from 0.0 to 1.0.
    pub fn on(&self, t: f32) -> f32 {
        self.curve.eval(t)
    }
}
