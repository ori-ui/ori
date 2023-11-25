use ori::prelude::*;

struct Data;

fn app(_data: &mut Data) -> impl View<Data> {
    let content = container(pad(pt(8.0), text("content")))
        .background(style(Palette::SECONDARY))
        .border_width(pt(2.0))
        .border_color(style(Palette::SECONDARY_DARKER))
        .border_radius(pt(6.0));

    let content = vstack![
        dropdown(button(text("header")), vscroll(content)),
        text("content")
    ];

    center(content)
}

fn main() {
    let window = WindowDescriptor::new().title("Dropdown (examples/dropdown.rs)");

    Launcher::new(Data).window(window, app).launch();
}
