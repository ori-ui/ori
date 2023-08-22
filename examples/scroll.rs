use ori::prelude::*;

#[derive(Default)]
struct Data {}

fn square() -> impl View<Data> {
    container(())
        .size(100.0)
        .background(style(Palette::SECONDARY))
}

fn app(_data: &mut Data) -> impl View<Data> {
    let scroll = height(
        400.0,
        vscroll(vstack![
            square(),
            square(),
            square(),
            square(),
            square(),
            square(),
            square(),
            square(),
            square()
        ]),
    );

    let button = button(text("hello"))
        .on_press(|_, _| {
            info!("hello");
        })
        .fancy(4.0);

    center(overlay![scroll, pad(em(0.5), bottom_right(button))])
}

fn main() {
    App::new(app, Data::default())
        .title("Scroll (examples/scroll.rs)")
        .run()
}
