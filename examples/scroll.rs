use ori::prelude::*;

fn square(index: usize) -> impl View {
    size(
        100.0,
        on_click(button(text("Click me")), move |_, _| {
            info!("clicked {}", index);
        }),
    )
}

fn ui() -> impl View {
    let scroll = height(
        400.0,
        vscroll(vstack![
            square(0),
            square(1),
            square(2),
            square(3),
            square(4),
            square(5),
            square(6),
            square(7),
            square(8)
        ]),
    );

    let button = button(text("hello")).fancy(4.0);
    center(zstack![scroll, pad(8.0, bottom_right(button))])
}

fn main() {
    ori::log::install().unwrap();

    let window = Window::new().title("Scroll (examples/scroll.rs)");
    ori::run_simple(window, ui).unwrap();
}
