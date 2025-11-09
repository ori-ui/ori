use ori_gtk4::{App, Effect, views::*};

struct Data {
    count: u32,
}

fn ui(data: &mut Data) -> impl Effect<Data> + use<> {
    let text = label(format!("clicked {} times", data.count));

    let button = button(text, |data: &mut Data| data.count += 1);

    window(button.halign(Align::Center).valign(Align::Center))
}

fn main() {
    let data = Data { count: 0 };

    App::new().theme("Adwaita-dark").run(data, ui).unwrap();
}
