use ori::prelude::*;

#[derive(Default)]
struct Data {
    counter: u32,
}

fn counter_button() -> impl View<Data> {
    let button = button(text("Click me!"))
        .fancy(4.0)
        .color(Image::gradient(0.0, &[hex("#020024"), hex("#00d4ff")]))
        .on_click(|_, data: &mut Data| data.counter += 1);

    alt("Counter Button", button)
}

fn app(data: &mut Data) -> impl View<Data> {
    center(vstack![
        rotate(data.counter as f32 * 0.3, counter_button()),
        alt(
            "Number of times the button has been clicked",
            text(format!("Clicked {} time(s)", data.counter))
        )
    ])
}

fn main() {
    App::new(app, Data::default())
        .title("Counter (examples/counter.rs)")
        .run();
}
