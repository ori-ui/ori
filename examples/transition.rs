use ori::prelude::*;

#[derive(Default)]
struct Data {
    on: bool,
}

fn app(data: &mut Data) -> impl View<Data> {
    let button = transition(ease(1.0), data.on, |_, t| {
        let text_color = Color::RED.mix(Color::GREEN, t);

        let label = text("Click me");
        button(label.color(text_color)).fancy(pt(4.0))
    });

    center(on_click(button, |_, data: &mut Data| data.on = !data.on))
}

fn main() {
    let window = WindowDescriptor::new().title("Transition (examples/transition.rs)");
    Launcher::new(Data::default()).window(window, app).launch();
}
