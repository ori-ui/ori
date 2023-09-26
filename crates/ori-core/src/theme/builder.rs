use std::fmt::Debug;

use super::Theme;

/// A builder for [`Theme`]s.
///
/// Themes might want to use the global theme to build themselves. For example
/// [`pt`](crate::style::pt) is used in a lot of places. So themes need to be rebuilt
/// when the scale factor or window size changes. The builders have the previous theme
/// as the global theme when building themselves.
#[derive(Default)]
pub struct ThemeBuilder {
    builders: Vec<Box<dyn FnMut() -> Theme>>,
}

impl ThemeBuilder {
    /// Create a new [`ThemeBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a theme builder.
    pub fn push(&mut self, builder: impl FnMut() -> Theme + 'static) {
        self.builders.push(Box::new(builder));
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
        f.debug_struct("ThemeBuilder").finish()
    }
}
