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
    let window = Window::new().title("Layout builder (examples/layout_builder.rs)");

    let app = App::build().window(window, app);

    ori::launch(app, Data).unwrap();
}
