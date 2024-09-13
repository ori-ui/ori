use ori::prelude::*;

fn ui() -> impl View {
    let click_me = transition_hovered(ease(0.2), |_, _, t| {
        let border = Color::RED.mix(Color::GREEN, t);

        button(text("Click me"))
            .border_width(2.0)
            .border_color(border)
            .fancy(4.0)
    });

    center(on_click(click_me, |_, _| info!("Clicked!")))
}

fn main() {
    ori::log::install().unwrap();

    let window = Window::new().title("Animate (examples/animate.rs)");
    ori::run_simple(window, ui).unwrap();
}
