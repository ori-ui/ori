use std::fmt::Debug;

use super::Theme;

/// A builder for [`Theme`]s.
///
/// Themes might want to use the global theme to build themselves.
/// The builders have the previous theme as the global theme when building themselves.
#[derive(Default)]
pub struct ThemeBuilder {
    builders: Vec<Box<dyn FnMut() -> Theme>>,
}

impl ThemeBuilder {
    /// Create a new [`ThemeBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of theme builders.
    pub fn len(&self) -> usize {
        self.builders.len()
    }

    /// Check if the theme builders are empty.
    pub fn is_empty(&self) -> bool {
        self.builders.is_empty()
    }

    /// Push a theme builder.
    pub fn push(&mut self, builder: impl FnMut() -> Theme + 'static) {
        self.builders.push(Box::new(builder));
    }

    /// Clear the theme builders.
    pub fn clear(&mut self) {
        self.builders.clear();
    }

    /// Build the theme.
    pub fn build(&mut self, theme: &mut Theme) {
        for builder in &mut self.builders {
            let new_theme = Theme::with_global(theme, builder);
            theme.extend(new_theme);
        }
    }
}

impl Debug for ThemeBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThemeBuilder")
            .field("builders", &self.builders.len())
            .finish()
    }
}
