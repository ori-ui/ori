use ori::prelude::*;

struct Data {
    color: Color,
}

impl Data {
    fn new() -> Self {
        Self {
            color: Color::okhsl(141.0, 0.88, 0.66),
        }
    }
}

fn ui(data: &mut Data) -> impl View<Data> {
    center(
        color_picker()
            .color(data.color)
            .on_input(|_, data: &mut Data, color| data.color = color),
    )
}

fn main() {
    let window = Window::new().title("Color Picker (examples/color_picker.rs)");

    let app = App::build().window(window, ui).style(Palette::light());

    ori::launch(app, Data::new()).unwrap();
}
