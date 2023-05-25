use ori::prelude::*;

const LONG_TEXT: &str = include_str!("long_text.txt");

fn ui(cx: Scope) -> Element {
    let counter = cx.signal(1);
    let checked = cx.signal(false);
    let knob_value = cx.signal(0.0);
    let long_text = cx.signal(String::from(LONG_TEXT));
    let text = cx.signal(String::new());

    let text_size = cx.memo(move || if checked.get() { Em(2.0) } else { Em(1.5) });

    let on_click = move |_: &PointerEvent| counter.set(counter.get() + 1);

    view! {
        <Div class="widget-gallery">
            <Div class="column">
                <Div class="row">
                    <Text text="Toggle me" style:font-size=trans(text_size.get(), 0.25) />
                    <Checkbox bind:checked=checked />
                </Div>

                <Button on:press=on_click>
                    { format!("Counter: {}", counter.get()) }
                </Button>

                <Image src="examples/images/image.jpg" />

                <TextInput bind:text=text />

                { format!("Input: {}", text.get()) }
            </Div>
            <Scroll style:max-height=Em(14.0)>
                <Div style:max-width=Em(8.0)>
                    <TextInput class="long-text" bind:text=long_text />
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
