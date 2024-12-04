use ori::prelude::*;

struct Data {
    text: String,
    multiline: bool,
}

impl Data {
    fn new() -> Self {
        Self {
            text: String::from("Hello"),
            multiline: false,
        }
    }

    fn toggle_multiline(&mut self) {
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
        .on_input(|_, data: &mut Data, text| data.text = text)
        .font_family(font)
        .multiline(data.multiline);

    container(pad(8.0, min_width(150.0, input)))
        .background(Theme::SURFACE)
        .border_radius(6.0)
}

fn clear_button() -> impl View<Data> {
    on_click(button(text("Clear")).fancy(4.0), |cx, data: &mut Data| {
        data.text.clear();
        cx.rebuild();
    })
}

fn multiline_checkbox(data: &mut Data) -> impl View<Data> {
    hstack![
        text("Multiline"),
        on_click(checkbox(data.multiline), |cx, data: &mut Data| {
            data.toggle_multiline();
            cx.rebuild();
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
    ori::log::install().unwrap();

    let window = Window::new().title("Text Input (examples/text_input.rs)");

    let app = App::build().window(window, app);

    ori::run(app, &mut Data::new()).unwrap();
}
