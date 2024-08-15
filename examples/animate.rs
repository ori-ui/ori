use ori::prelude::*;

fn app() -> impl View {
    let click_me = transition_hot(ease(0.2), |_, _, t| {
        let border = Color::RED.mix(Color::GREEN, t);

        button(text("Click me"))
            .border_width(2.0)
            .border_color(border)
            .fancy(4.0)
    });

    center(on_click(click_me, |_, _| info!("Clicked!")))
}

fn main() {
    let window = Window::new().title("Animate (examples/animate.rs)");
    let app = App::build().window(window, app);
    ori::launch(app, ()).unwrap();
}
