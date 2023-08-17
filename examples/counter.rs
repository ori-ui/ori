use ori::prelude::*;

#[derive(Default)]
struct Data {
    counter: f32,
}

fn counter_button() -> impl View<Data> {
    button(text("Click me!"), |data: &mut Data| data.counter += 1.0).fancy(4.0)
}

fn app(data: &mut Data) -> impl View<Data> {
    align_center(
        vstack![
            counter_button(),
            text(format!("Clicked {} time(s)", data.counter))
        ]
        .align_items(AlignItems::Center),
    )
}

fn main() {
    App::new(app, Data::default())
        .title("Counter (examples/counter.rs)")
        .run();
}
