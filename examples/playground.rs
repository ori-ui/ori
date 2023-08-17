use ori::prelude::*;

#[derive(Default)]
struct Data {
    counter: f32,
}

fn counter_button(data: &mut Data) -> impl View<Data> {
    button(text(format!("{}", data.counter)), |data: &mut Data| {
        data.counter += 1.0
    })
    .fancy(4.0)
    .padding(2.0)
}

fn app(data: &mut Data) -> impl View<Data> {
    let mut buttons = Vec::new();

    for _ in 0..2000 {
        buttons.push(counter_button(data));
    }

    hstack(buttons).gap(2.0)
}

fn main() {
    App::new(app, Data::default()).run();
}
