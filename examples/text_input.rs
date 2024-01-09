use ori::prelude::*;

#[derive(Default)]
struct Data {
    text: String,
    multiline: bool,
}

impl Data {
    fn toogle_multiline(&mut self) {
        self.multiline = !self.multiline;
    }
}

fn input(data: &mut Data) -> impl View<Data> {
    let input = text_input()
        .text(&data.text)
        .on_change(|_, data: &mut Data, text| data.text = text)
        .multiline(data.multiline);

    container(pad(8.0, min_width(150.0, input)))
        .background(style(Palette::SECONDARY))
        .border_radius(6.0)
}

fn clear_button() -> impl View<Data> {
    on_click(button(text("Clear")).fancy(4.0), |_, data: &mut Data| {
        data.text.clear()
    })
}

fn multiline_checkbox(data: &mut Data) -> impl View<Data> {
    hstack![
        text("Multiline"),
        checkbox(data.multiline).on_press(|_, data: &mut Data| data.toogle_multiline())
    ]
}

fn app(data: &mut Data) -> impl View<Data> {
    center(
        vstack![
            multiline_checkbox(data),
            hstack![input(data), clear_button()],
        ]
        .align_items(Align::Start),
    )
}

fn main() {
    let window = WindowDescriptor::new().title("Text Input (examples/text_input.rs)");
    Launcher::new(Data::default()).window(window, app).launch();
}
