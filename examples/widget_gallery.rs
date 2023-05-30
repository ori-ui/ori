use ori::prelude::*;

const LONG_TEXT: &str = include_str!("long_text.txt");

fn popup_ui(cx: Scope) -> Element {
    // use an Atom to store the counter value, so that it persists when popup closes
    static COUNTER: Atom<i32> = atom!(0);

    view! {
        <Div class="widget-gallery">
            <Div class="popup">
                <Button on:click=move |_| *COUNTER.modify() += 1>
                    "Click me!"
                </Button>
                { format!("Clicked {} times", COUNTER.get()) }
            </Div>
        </Div>
    }
}

fn ui(cx: Scope) -> Element {
    let counter = cx.signal(1);
    let checked = cx.signal(false);
    let knob_value = cx.signal(0.0);
    let long_text = cx.signal(String::from(LONG_TEXT));
    let text = cx.signal(String::new());

    // popup state
    let popup_window = cx.signal(None);

    // when popup_open changes, open or close the popup window
    let toggle_popup = move |_: &PointerEvent| {
        if popup_window.get().is_none() {
            let window = Window::new().title("Widget Gallery Popup").size(300, 300);
            let id = cx.open_window(window, popup_ui);

            popup_window.set(Some(id));
        } else {
            if let Some(id) = popup_window.get() {
                cx.emit_event(CloseWindow::window(id));
                popup_window.set(None);
            }
        }
    };

    cx.on_event(move |event| {
        if event.is::<WindowClosedEvent>() {
            popup_window.set(None);
        }
    });

    let text_size = cx.memo(move || if checked.get() { Em(2.0) } else { Em(1.5) });
    let on_click = move |_: &PointerEvent| *counter.modify() += 1;

    view! {
        <Div class="widget-gallery">
            <Div class="column">
                <Div class="row">
                    <Text text="Toggle me" style:font-size=trans(text_size.get(), 0.25) />
                    <Checkbox bind:checked=checked />
                </Div>

                <Div class="row" style:justify-content=JustifyContent::End>
                    "Popup"
                    <Checkbox checked=popup_window.get().is_some() on:click=toggle_popup />
                </Div>

                <Button on:click=on_click>
                    { format!("Counter: {}", counter.get()) }
                </Button>

                <Image src="examples/images/image.jpg" />

                <Text text=text.get() />

                { format!("Input: {}", text.get()) }
            </Div>
            <Scroll style:max-height=Em(14.0)>
                <Div style:max-width=Em(8.0)>
                    <Text class="long-text" text=long_text.get() />
                </Div>
            </Scroll>
            <Div class="column">
                <Knob bind:value=knob_value max=2.0 />
                <Text style:text-align=TextAlign::Center
                    text=format!("{:.2}", knob_value.get()) />
            </Div>
            <Slider style:direction=Axis::Vertical style:height=Em(10.0) min=-1.0 bind:value=knob_value />
            <Div class="column">
                <Slider />
                <Slider />
                <Slider />
                <Slider />
                <Slider />
                <Slider />
                <Slider />
            </Div>
        </Div>
    }
}

fn main() {
    App::new(ui) // create a new app with the ui function
        .title("Widget Gallery (examples/widget_gallery.rs)") // set the window title
        .style("examples/style/widget-gallery.css") // load a custom stylesheet
        .run(); // run the app
}
