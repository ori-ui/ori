use ori::prelude::*;

struct Data;

fn app(_data: &mut Data) -> impl View<Data> {
    let mut buttons = Vec::with_capacity(1000);

    for _ in 0..1000 {
        buttons.push(on_click(
            button(text("B")).padding(4.0).border_radius(4.0),
            |_, _| {},
        ));
    }

    hstack(buttons).wrap(true)
}

fn main() {
    let window = WindowDescriptor::new().title("Thousand Buttons (examples/thousand_buttons.rs)");
    Launcher::new(Data).window(window, app).launch()
}
