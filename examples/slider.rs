use ori::prelude::*;

struct Data {
    font_size: f32,
}

impl Data {
    fn new() -> Self {
        Self { font_size: 16.0 }
    }
}

fn ui(data: &mut Data) -> impl View<Data> {
    center(vstack![
        slider(data.font_size)
            .on_input(|_, data: &mut Data, font_size| data.font_size = font_size)
            .range(8.0..=32.0),
        text("Hello, World!").font_size(data.font_size),
    ])
}

fn main() {
    let window = Window::new().title("Slider (examples/slider.rs)");

    let app = App::build().window(window, ui);

    ori::launch(app, &mut Data::new()).unwrap();
}
