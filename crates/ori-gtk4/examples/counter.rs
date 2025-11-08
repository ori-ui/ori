use ori_gtk4::{App, SideEffect, views::*};

struct Data {
    count: u32,
}

fn ui(data: &mut Data) -> impl SideEffect<Data> + use<> {
    let text = label(format!("clicked {} times", data.count));

    let button = button(text, |data: &mut Data| data.count += 1);

    window(center(button))
}

fn main() {
    let data = Data { count: 0 };

    App::new().theme("Adwaita-dark").run(data, ui).unwrap();
}
