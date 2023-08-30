use ori::prelude::*;

#[derive(Default)]
struct Data {
    counter: f32,
}

fn counter_button() -> impl View<Data> {
    let button = button(text("Click me!"))
        .on_press(|_, data: &mut Data| data.counter += 1.0)
        .fancy(4.0);

    alt("Counter Button", button)
}

fn app(data: &mut Data) -> impl View<Data> {
    center(vstack![
        counter_button(),
        text(format!("Clicked {} time(s)", data.counter))
    ])
}

fn main() {
    App::new(app, Data::default())
        .title("Counter (examples/counter.rs)")
        .run();
}
