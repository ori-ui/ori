use ori_gtk4::{App, Layer, View, Window, label};

struct Data {}

fn ui(_data: &mut Data) -> impl View<Data> + use<> {
    label("hello")
}

fn main() {
    let data = Data {};

    let window = Window::new()
        .layer(Layer::Overlay)
        .height(200)
        .anchor_left(true)
        .anchor_bottom(true)
        .anchor_right(true)
        .margin_left(40)
        .margin_right(40)
        .margin_top(20);

    App::new().window(window, ui).run(data).unwrap();
}
