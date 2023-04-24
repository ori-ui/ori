use std::sync::{Arc, Mutex};

use ori::prelude::*;

const LONG_TEXT: &str = include_str!("long_text.txt");

struct Droop<'a> {
    item: Arc<Mutex<Option<&'a mut Vec<String>>>>,
}

impl<'a> Drop for Droop<'a> {
    fn drop(&mut self) {
        self.item
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .push(String::from("test"));
    }
}

fn ui(cx: Scope) -> impl View {
    let counter = cx.signal(1);
    let checked = cx.signal(false);
    let knob_value = cx.signal(0.0);
    let text = cx.signal(String::new());

    let font_size = cx.memo(|| if *checked.get() { 32 } else { 24 });

    checked.track();

    view! {
        <Div class="widget-gallery">
            <Div class="column">
                <Div class="row">
                    <Text text="Toggle me" style:font-size=trans(font_size, 0.5) />
                    <Checkbox bind:checked=checked />
                </Div>

                <Button on:press=|_| *counter.modify() += 1>
                    <Text text=format!("Counter: {}", counter.get()) />
                </Button>

                <Image src="examples/images/image.jpg" />

                <TextInput bind:text=text />

                <Text text=format!("Input: {}", text.get()) />
            </Div>
            <Scroll style:max-height=Em(14.0)>
                <Div style:max-width=Em(8.0)>
                    <Text text=LONG_TEXT />
                </Div>
            </Scroll>
            <Div class="column">
                <Knob bind:value=knob_value />
                <Text style:text-align=TextAlign::Center
                    text=format!("{:.2}", knob_value.get()) />
            </Div>
        </Div>
    }
}

fn main() {
    App::new(|cx| ui(cx)) // create a new app with the ui function
        .title("Widget Gallery (examples/widget_gallery.rs)") // set the window title
        .night_theme()
        .style("examples/style/widget-gallery.css") // load a custom stylesheet
        .run(); // run the app
}
