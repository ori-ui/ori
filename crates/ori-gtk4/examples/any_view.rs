use ori_gtk4::{App, SideEffect, views::*};

struct Data {
    toggle: bool,
    text: String,
}

fn ui(data: &mut Data) -> impl SideEffect<Data> + use<> {
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

    let view = center(vline![toggle, view].spacing(10));

    window(view).title("any view")
}

fn main() {
    let data = Data {
        toggle: false,
        text: String::new(),
    };

    App::new().run(data, ui).unwrap();
}
