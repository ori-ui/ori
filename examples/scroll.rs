use ori::prelude::*;

#[derive(Default)]
struct Data {}

fn square() -> impl View<Data> {
    container(())
        .size(100.0)
        .background(style(Palette::SECONDARY))
}

fn app(_data: &mut Data) -> impl View<Data> {
    center(height(
        400.0,
        scroll(vstack![
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
    ))
}

fn main() {
    App::new(app, Data::default())
        .title("Scroll (examples/scroll.rs)")
        .run()
}
