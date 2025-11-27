use ori_gtk4::prelude::*;

struct Data {
    text: String,
}

fn ui(data: &mut Data) -> impl Effect<Data> + use<> {
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

    let view = vbox((text, clear.halign(Align::Center)))
        .spacing(10)
        .halign(Align::Center)
        .valign(Align::Center);

    window(view).title("text input")
}

fn main() {
    let data = Data {
        text: String::new(),
    };

    App::new().run(data, ui).unwrap();
}
