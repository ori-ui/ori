use ori::prelude::*;

#[derive(Default)]
struct Data {}

fn app(_data: &mut Data) -> impl View<Data> {
    let click_me = animate_hot(ease(0.2), |_, t| {
        let border = Color::RED.mix(Color::GREEN, t);

        button(text("Click me"))
            .border_width(pt(2.0))
            .border_color(border)
            .fancy(pt(4.0))
    });

    center(on_click(click_me, |_, _| info!("Clicked!")))
}

fn main() {
    App::new(app, Data::default())
        .title("Animate (examples/animate.rs)")
        .run();
}
