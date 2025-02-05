use ori::prelude::*;

fn ui() -> impl View {
    layout(|_, space| {
        build(move |cx, _| {
            let box_count = if space.max.width > 800.0 { 2 } else { 1 };

            let mut boxes = hstack_vec().gap(100.0);

            for _ in 0..box_count {
                boxes.push(background(cx.theme().primary, size(100.0, ())));
            }

            center(vstack![text!("Space: {}", space.max), boxes].gap(50.0))
        })
    })
}

fn main() {
    ori::log::install().unwrap();

    let window = Window::new().title("Layout Example (examples/layout.rs)");

    ori::run_simple(window, ui).unwrap();
}
