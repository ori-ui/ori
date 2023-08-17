use ori::prelude::*;

#[derive(Default)]
struct Data {
    text: String,
}

fn app(_data: &mut Data) -> impl View<Data> {
    let input = text_input(|data: &mut Data| &mut data.text);

    align_center(input)
}

fn main() {
    App::new(app, Data::default()).run();
}
