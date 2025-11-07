use ori_gtk4::{App, View, Window, views::*};

struct Data {
    toggle: bool,
    text: String,
}

fn ui(data: &mut Data) -> impl View<Data> + use<> {
    let toggle = button(label("toggle"), |data: &mut Data| {
        data.toggle = !data.toggle;
    });

    let view = if data.toggle {
        let view = hline![
            checkbox(|_, checked| println!("checked: {checked}")),
            label("checkbox"),
        ]
        .spacing(4);

        any(view)
    } else {
        let view = entry()
            .text(&data.text)
            .on_change(|data: &mut Data, text| data.text = text);

        any(view)
    };

    center(vline![toggle, view].spacing(10))
}

fn main() {
    let data = Data {
        toggle: false,
        text: String::new(),
    };

    let window = Window::new().title("any view");

    App::new().window(window, ui).run(data).unwrap();
}
