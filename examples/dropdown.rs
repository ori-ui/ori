use ori::prelude::*;

fn ui() -> impl View {
    let content = container(pad(8.0, text("content")))
        .border_width(2.0)
        .border_radius(6.0);

    let content = vstack![
        dropdown(button(text("header")), vscroll(content)),
        text("content")
    ];

    center(content)
}

fn main() {
    let window = Window::new().title("Dropdown (examples/dropdown.rs)");
    ori::launch_simple(window, ui).unwrap();
}
