use std::str::FromStr;

use ori::prelude::*;
use style::ParseError;

struct Data {
    text: String,
    base: Styles,
    styles: Styles,
    error: Option<ParseError>,
}

fn ui(data: &mut Data) -> impl View<Data> {
    let title = text!("An example of using styling");
    let title = tooltip(title, "This has the class 'title'");
    let title = class("title", title);

    let subtitle = text!("Styling is easy and powerful");
    let subtitle = tooltip(subtitle, "This has the class 'subtitle'");
    let subtitle = class("subtitle", subtitle);

    let style = text_input()
        .on_input(|cx, data: &mut Data, text| {
            data.text = text;

            match Styles::from_str(&data.text) {
                Ok(styles) => {
                    data.styles = styles;
                    data.error = None;
                    cx.rebuild();
                }

                Err(err) => {
                    data.error = Some(err);
                    cx.rebuild();
                }
            }
        })
        .text(&data.text)
        .multiline(true);

    let style = tooltip(style, "This has the class 'style'");
    let style = vscroll(padded(style));
    let style = max_height(500.0, style);
    let style = container(style);
    let style = class("style", style);

    let error = if let Some(err) = &data.error {
        let error = text!("{}", err);
        let error = tooltip(error, "This has the class 'error'");
        let error = class("error", error);

        Some(error)
    } else {
        None
    };

    let view = center(vstack![title, subtitle, style, error]);

    with_styles(data.styles.clone(), view)
}

struct Delegate;

impl AppDelegate<Data> for Delegate {
    fn init(&mut self, cx: &mut DelegateCx<Data>, data: &mut Data) {
        data.base = cx.context::<Styles>().clone();
    }

    fn event(&mut self, _cx: &mut DelegateCx<Data>, _data: &mut Data, _event: &Event) -> bool {
        false
    }
}

fn main() {
    ori::log::install().unwrap();

    let window = Window::new().title("Style (examples/style.rs)");

    let app = App::build().window(window, ui).delegate(Delegate);

    let mut data = Data {
        text: String::from(include_str!("style.oss")),
        base: Styles::new(),
        styles: Styles::from_str(include_str!("style.oss")).unwrap(),
        error: None,
    };

    ori::run(app, &mut data).unwrap();
}
