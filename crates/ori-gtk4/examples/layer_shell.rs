use ori_gtk4::{App, Effect, views::*};

struct Data {}

fn ui(_data: &mut Data) -> impl Effect<Data> + use<> {
    window(label("hello"))
        .layer(Layer::Overlay)
        .height(200)
        .anchor_left(true)
        .anchor_bottom(true)
        .anchor_right(true)
        .margin_left(40)
        .margin_right(40)
        .margin_top(20)
}

fn main() {
    let data = Data {};

    App::new().run(data, ui).unwrap();
}
