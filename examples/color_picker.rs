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
    let picker = color_picker()
        .color(data.color)
        .on_input(|cx, data: &mut Data, color| {
            data.color = color;
            cx.request_rebuild();
        });

    let color = background(data.color, height(30.0, ())).border_radius(8.0);

    center(vstack![picker, color].align(Align::Stretch).gap(10.0))
}

fn main() {
    let window = Window::new().title("Color Picker (examples/color_picker.rs)");

    let app = App::build().window(window, ui).style(Palette::light());

    ori::launch(app, Data::new()).unwrap();
}
