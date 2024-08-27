use ori::prelude::*;

fn ui() -> impl View {
    let content = vstack![
        text("Hello, world!"),
        text("Hello, world!"),
        text("Hello, world!"),
        text("Hello, world!"),
    ];

    let collapsing = collapsing(trigger(text("Click me!")), content);

    center(collapsing)
}

fn main() {
    let window = Window::new().title("Collapsing (examples/collapsing.rs)");
    ori::run_simple(window, ui).unwrap();
}
