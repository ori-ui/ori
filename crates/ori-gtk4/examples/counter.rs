use ori_gtk4::{App, View, Window, views::*};

struct Data {
    count: u32,
}

fn ui(data: &mut Data) -> impl View<Data> + use<> {
    let text = label(format!("clicked {} times", data.count));

    let button = button(text, |data: &mut Data| data.count += 1);

    center(size(80, 80, button))
}

fn main() {
    let data = Data { count: 0 };

    let window = Window::new().title("counter");

    App::new().theme("Adwaita-dark").window(window, ui).run(data).unwrap();
}
