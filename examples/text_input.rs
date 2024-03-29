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
    let font = if data.multiline {
        FontFamily::Monospace
    } else {
        FontFamily::SansSerif
    };

    let input = text_input()
        .text(&data.text)
        .on_change(|_, data: &mut Data, text| data.text = text)
        .font_family(font)
        .multiline(data.multiline);

    container(pad(8.0, min_width(150.0, input)))
        .background(palette().secondary())
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
        on_click(checkbox(data.multiline), |_, data: &mut Data| {
            data.toogle_multiline();
        })
    ]
}

fn app(data: &mut Data) -> impl View<Data> {
    center(
        vstack![
            multiline_checkbox(data),
            hstack![input(data), clear_button()],
        ]
        .align(Align::Start),
    )
}

fn main() {
    let window = WindowDescriptor::new().title("Text Input (examples/text_input.rs)");
    Launcher::new(Data::default()).window(window, app).launch();
}
