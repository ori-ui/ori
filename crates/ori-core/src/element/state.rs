use std::time::Instant;

use ori_graphics::Rect;
use uuid::Uuid;

use crate::{
    AvailableSpace, Context, FromStyleAttribute, Margin, Padding, Style, StyleAttribute,
    StyleSelector, StyleSpecificity, StyleStates, StyleTransition, TransitionStates,
};

/// An element identifier. This uses a UUID to ensure that elements are unique.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ElementId {
    uuid: Uuid,
}

impl ElementId {
    /// Create a new element identifier, using uuid v4.
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }

    /// Gets the inner uuid.
    pub const fn uuid(self) -> Uuid {
        self.uuid
    }
}

impl Default for ElementId {
    fn default() -> Self {
        Self::new()
    }
}

/// The state of a element, which is used to store information about the element.
///
/// This should almost never be used directly, and instead should be used through the [`Element`]
/// struct.
#[derive(Clone, Debug)]
pub struct ElementState {
    pub id: ElementId,
    pub margin: Margin,
    pub padding: Padding,
    pub local_rect: Rect,
    pub global_rect: Rect,
    pub active: bool,
    pub focused: bool,
    pub hovered: bool,
    pub last_draw: Instant,
    pub style: Style,
    pub needs_layout: bool,
    pub available_space: AvailableSpace,
    pub transitions: TransitionStates,
}

impl Default for ElementState {
    fn default() -> Self {
        Self {
            id: ElementId::new(),
            margin: Margin::ZERO,
            padding: Padding::ZERO,
            local_rect: Rect::ZERO,
            global_rect: Rect::ZERO,
            active: false,
            focused: false,
            hovered: false,
            last_draw: Instant::now(),
            style: Style::default(),
            needs_layout: true,
            available_space: AvailableSpace::ZERO,
            transitions: TransitionStates::new(),
        }
    }
}

impl ElementState {
    /// Create a new [`ElementState`] with the given style.
    pub fn new(style: Style) -> Self {
        Self {
            style,
            ..Default::default()
        }
    }

    /// Propagate the [`ElementState`] up to the parent.
    ///
    /// This is called before events are propagated.
    pub fn propagate_up(&mut self, parent: &mut ElementState) {
        self.global_rect = self.local_rect.translate(parent.global_rect.min);
    }

    /// Propagate the [`ElementState`] down to the child.
    ///
    /// This is called after events are propagated.
    pub fn propagate_down(&mut self, child: &mut ElementState) {
        self.needs_layout |= child.needs_layout;
    }

    /// Returns the [`StyleStates´].
    pub fn style_states(&self) -> StyleStates {
        let mut states = StyleStates::new();

        if self.active {
            states.push("active");
        }

        if self.focused {
            states.push("focus");
        }

        if self.hovered {
            states.push("hover");
        }

        states
    }

    /// Returns the [`StyleSelector`].
    pub fn selector(&self) -> StyleSelector {
        StyleSelector {
            element: self.style.element.map(Into::into),
            classes: self.style.classes.clone(),
            states: self.style_states(),
        }
    }

    /// Returns the time in seconds since the last draw.
    pub fn delta_time(&self) -> f32 {
        self.last_draw.elapsed().as_secs_f32()
    }

    /// Gets the style attribute for the given key.
    pub fn get_style_attribyte(
        &mut self,
        cx: &mut impl Context,
        key: &str,
    ) -> Option<StyleAttribute> {
        self.get_style_attribute_specificity(cx, key)
            .map(|(attribute, _)| attribute)
    }

    /// Gets the style attribute and specificity for the given key.
    pub fn get_style_attribute_specificity(
        &mut self,
        cx: &mut impl Context,
        key: &str,
    ) -> Option<(StyleAttribute, StyleSpecificity)> {
        if let Some(attribute) = self.style.attributes.get(key) {
            return Some((attribute.clone(), StyleSpecificity::INLINE));
        }

        let selectors = cx.selectors().clone().with(self.selector());
        let hash = selectors.hash();

        if let Some(result) = cx.style_cache().get_attribute(hash, key) {
            return result;
        }

        let stylesheet = cx.stylesheet();

        match stylesheet.get_attribute_specificity(&selectors, key) {
            Some((attribute, specificity)) => {
                (cx.style_cache_mut()).insert(hash, attribute.clone(), specificity);
                Some((attribute, specificity))
            }
            None => {
                cx.style_cache_mut().insert_none(hash, key);
                None
            }
        }
    }

    /// Gets the style attribute for the given key, and converts it to the given type.
    pub fn get_style_specificity<T: FromStyleAttribute + 'static>(
        &mut self,
        cx: &mut impl Context,
        key: &str,
    ) -> Option<(T, StyleSpecificity)> {
        let (attribute, specificity) = self.get_style_attribute_specificity(cx, key)?;
        let value = T::from_attribute(attribute.value().clone())?;
        let transition = attribute.transition();

        Some((self.transition(key, value, transition), specificity))
    }

    /// Gets the style attribute for the given key, and converts it to the given type.
    pub fn get_style<T: FromStyleAttribute + 'static>(
        &mut self,
        cx: &mut impl Context,
        key: &str,
    ) -> Option<T> {
        self.get_style_specificity(cx, key).map(|(value, _)| value)
    }

    /// Gets the style attribute for the given key, and converts it to the given type.
    pub fn style<T: FromStyleAttribute + Default + 'static>(
        &mut self,
        cx: &mut impl Context,
        key: &str,
    ) -> T {
        self.get_style(cx, key).unwrap_or_default()
    }

    /// Gets the style for a group of keys.
    pub fn style_group<T: FromStyleAttribute + Default + 'static>(
        &mut self,
        cx: &mut impl Context,
        keys: &[&str],
    ) -> T {
        let mut specificity = None;
        let mut result = None;

        for key in keys {
            if let Some((v, s)) = self.get_style_specificity(cx, key) {
                if specificity.is_none() || s > specificity.unwrap() {
                    specificity = Some(s);
                    result = Some(v);
                }
            }
        }

        result.unwrap_or_default()
    }

    /// Transition a value.
    ///
    /// If the value is an [`f32`], or a [`Color`](ori_graphics::Color), then it will be transitioned.
    pub fn transition<T: 'static>(
        &mut self,
        name: &str,
        mut value: T,
        transition: Option<StyleTransition>,
    ) -> T {
        (self.transitions).transition_any(name, &mut value, transition);
        value
    }

    /// Update the transitions.
    pub fn update_transitions(&mut self) -> bool {
        self.transitions.update(self.delta_time())
    }

    /// Returns `true` if the available space has changed.
    pub fn space_changed(&mut self, space: AvailableSpace) -> bool {
        self.available_space != space
    }

    /// Updates `self.last_draw` to the current time.
    pub(crate) fn draw(&mut self) {
        self.last_draw = Instant::now();
    }
}
