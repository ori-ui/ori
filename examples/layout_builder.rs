use ori::prelude::*;

fn ui() -> impl View {
    center(layout_builder(|_, _, space| {
        vstack![
            text!("I have {}..{}, space to work with.", space.min, space.max),
            text!("Try resizing the window!"),
        ]
    }))
}

fn main() {
    let window = Window::new().title("Layout builder (examples/layout_builder.rs)");
    ori::launch_simple(window, ui).unwrap();
}
