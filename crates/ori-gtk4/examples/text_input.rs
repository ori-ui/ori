use ori_gtk4::{App, View, Window, button, center, column, entry, label};

struct Data {
    text: String,
}

fn ui(data: &mut Data) -> impl View<Data> + use<> {
    let text = entry()
        .text(&data.text)
        .placeholder("Try typing here!")
        .on_change(|data: &mut Data, text| {
            data.text = text;
        });

    let clear = button(label("Clear"), |data: &mut Data| {
        data.text.clear();
    });

    center(column![text, clear].spacing(10))
}

fn main() {
    let data = Data {
        text: String::new(),
    };

    let window = Window::new().title("text input");

    App::new().window(window, ui).run(data).unwrap();
}
