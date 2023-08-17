use crate::{style, Color, Palette, Theme, Transition};

style! {
    pub button {
        const TRANSITION: Transition = Transition::ease(0.1);
        const COLOR: Color = Palette::PRIMARY;
        const BORDER_RADIUS: [f32; 4] = [8.0; 4];
        const BORDER_WIDTH: [f32; 4] = [0.0; 4];
        const BORDER_COLOR: Color = Color::TRANSPARENT;
    }
}

style! {
    pub container {
        const BACKGROUND: Color = Color::TRANSPARENT;
        const BORDER_RADIUS: [f32; 4] = [0.0; 4];
        const BORDER_WIDTH: [f32; 4] = [0.0; 4];
        const BORDER_COLOR: Color = Color::TRANSPARENT;
    }
}

impl Theme {
    pub fn builtin() -> Self {
        let mut theme = Self::new();

        theme.extend(button::default_theme());
        theme.extend(container::default_theme());

        theme
    }
}
