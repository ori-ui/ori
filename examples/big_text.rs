use ori::prelude::*;

fn ui() -> impl View {
    center(text("Hello, World!").font_size(100.0).font_family("Roboto"))
}

fn main() {
    let window = Window::new().title("Hello, World! (examples/hello_world.rs)");
    ori::run_simple(window, ui).unwrap();
}
