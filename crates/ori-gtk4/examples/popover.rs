use ori_gtk4::prelude::*;

struct Data {}

fn ui(_: &mut Data) -> impl Effect<Data> + use<> {
    const KEY: Key = Key::new("example.popover");

    let button = popover(
        KEY,
        button(
            label("Click me to open popover"),
            |_| Event::new(PopoverCommand::Popup, KEY),
        ),
        label("I am a popover!"),
    );

    window(button.halign(Align::Center).valign(Align::Center))
}

fn main() {
    let data = Data {};

    App::new().run(data, ui).unwrap();
}
