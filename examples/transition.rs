use ori::prelude::*;

#[derive(Default)]
struct Data {
    on: bool,
}

fn ui(data: &mut Data) -> impl View<Data> {
    let button = transition(ease(1.0), data.on, |_, _, t| {
        let text_color = Color::RED.mix(Color::GREEN, t);

        let label = text("Click me");
        button(label.color(text_color)).fancy(4.0)
    });

    center(on_click(button, |_, data: &mut Data| data.on = !data.on))
}

fn main() {
    let window = Window::new().title("Transition (examples/transition.rs)");

    let app = App::build().window(window, ui);

    ori::launch(app, Data::default()).unwrap();
}
