use ori::prelude::*;

#[derive(Default)]
struct Data {}

fn app(_data: &mut Data) -> impl View<Data> {
    let click_me = transition_hot(ease(0.2), |_, _, t| {
        let border = Color::RED.mix(Color::GREEN, t);

        button(text("Click me"))
            .color(gradient(0.0, &[palette().primary(), palette().accent()]))
            .border_width(2.0)
            .border_color(border)
            .fancy(4.0)
    });

    center(on_click(click_me, |_, _| info!("Clicked!")))
}

fn main() {
    let window = WindowDescriptor::new().title("Animate (examples/animate.rs)");
    Launcher::new(Data::default()).window(window, app).launch();
}
