use ike::Color;

#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    pub surface: Color,
    pub outline: Color,
    pub primary: Color,
    pub error:   Color,
    pub success: Color,
    pub info:    Color,
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            surface: Color::hex("353535"),
            outline: Color::hex("808080"),
            primary: Color::hex("8caaee"),
            error:   Color::RED,
            success: Color::GREEN,
            info:    Color::BLUE,
        }
    }
}
