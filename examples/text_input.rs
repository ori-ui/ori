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
        .bind_text(|data: &mut Data| &mut data.text)
        .on_submit(|_, _, text| info!("Submitted: {}", text))
        .multiline(data.multiline)
        .min_width(150.0);

    pad(8.0, input)
        .background(style(Palette::SECONDARY))
        .border_radius(6.0)
}

fn clear_button() -> impl View<Data> {
    button(text("Clear"))
        .on_press(|_, data: &mut Data| data.text.clear())
        .fancy(4.0)
}

fn multiline_checkbox(data: &mut Data) -> impl View<Data> {
    hstack![
        text("Multiline"),
        checkbox(data.multiline).on_press(|_, data: &mut Data| data.toogle_multiline())
    ]
    .align_items(Align::Center)
}

fn app(data: &mut Data) -> impl View<Data> {
    center(vstack![
        multiline_checkbox(data),
        hstack![input(data), clear_button()],
    ])
}

fn main() {
    App::new(app, Data::default())
        .title("Text Input (examples/text_input.rs)")
        .run();
}
