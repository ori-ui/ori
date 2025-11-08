use ori_gtk4::{App, SideEffect, views::*};

struct Data {
    text: String,
}

fn ui(data: &mut Data) -> impl SideEffect<Data> + use<> {
    let text = entry()
        .text(&data.text)
        .placeholder("Try typing here!")
        .on_change(|data: &mut Data, text| data.text = text)
        .on_submit(|data: &mut Data, text| {
            data.text.clear();
            println!("Text inputted: {}", text);
        });

    let clear = button(label("Clear"), |data: &mut Data| {
        data.text.clear();
    });

    let view = center(vline![text, center(clear)].spacing(10));

    window(view).title("text input")
}

fn main() {
    let data = Data {
        text: String::new(),
    };

    App::new().run(data, ui).unwrap();
}
