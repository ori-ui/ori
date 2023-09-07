use ori::prelude::*;

struct Data {}

fn app(_data: &mut Data) -> impl View<Data> {
    let mut buttons = Vec::with_capacity(1000);

    for _ in 0..1000 {
        buttons.push(
            button(text("B"))
                .padding(pt(4.0))
                .border_radius(pt(4.0))
                .on_click(|_, _| {}),
        );
    }

    hstack(buttons).wrap(true)
}

fn main() {
    App::new(app, Data {})
        .title("Thousand Buttons (examples/thousand_buttons.rs)")
        .run()
}
