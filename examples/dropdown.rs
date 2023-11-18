use ori::prelude::*;

struct Data;

fn app(_data: &mut Data) -> impl View<Data> {
    let header = button(text("header"));
    let content = container(pad(pt(8.0), text("content")))
        .border_width(pt(2.0))
        .border_color(style(Palette::SECONDARY_DARKER))
        .border_radius(pt(6.0));

    let content = dropdown(header, content).toggle(true);

    center(content)
}

fn main() {
    Launcher::new(app, Data)
        .title("Dropdown (examples/dropdown.rs)")
        .launch();
}
