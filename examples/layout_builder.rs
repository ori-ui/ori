use ori::prelude::*;

struct Data;

fn app(_data: &mut Data) -> impl View<Data> {
    center(layout_builder(|_, _, space| {
        vstack![
            text!("I have {}..{}, space to work with.", space.min, space.max),
            text!("Try resizing the window!"),
        ]
    }))
}

fn main() {
    let window = WindowDescriptor::new().title("Layout builder (examples/layout_builder.rs)");

    Launcher::new(Data).window(window, app).launch();
}
