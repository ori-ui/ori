use ori::prelude::*;

const LONG_TEXT: &str = include_str!("long_text.txt");

fn ui(cx: Scope) -> impl View {
    let counter = cx.signal(1);
    let checked = cx.signal(false);
    let knob_value = cx.signal(0.0);
    let long_text = cx.signal(String::from(LONG_TEXT));
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
                    <TextInput class="long-text" bind:text=long_text />
                </Div>
            </Scroll>
            <Div class="column">
                <Knob bind:value=knob_value />
                <Text style:text-align=TextAlign::Center
                    text=format!("{:.2}", knob_value.get()) />
            </Div>
            <Slider style:direction=Axis::Vertical style:height=Em(10.0) value=0.5/>
            <Div class="column">
                <Slider/>
                <Slider/>
                <Slider/>
                <Slider/>
                <Slider/>
                <Slider/>
                <Slider/>
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
