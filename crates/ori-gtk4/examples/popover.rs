use ori_gtk4::prelude::*;

struct Data {}

fn ui(_: &mut Data) -> impl Effect<Data> + use<> {
    const ID: ViewId = ViewId::new("example.popover");

    let button = popover(
        ID,
        button(
            label("Click me to open popover"),
            |_| Event::new(PopoverCommand::Popup, ID),
        ),
        label("I am a popover!"),
    );

    window(button.halign(Align::Center).valign(Align::Center))
}

fn main() {
    let data = Data {};

    App::new().run(data, ui).unwrap();
}
