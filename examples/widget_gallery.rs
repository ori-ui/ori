use ily::prelude::*;

fn ui<'a>(cx: Scope<'a>) -> impl View {
    let counter = cx.signal(1);
    let checked = cx.signal(false);
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
            </Div>
        </Div>
    }
}

fn main() {
    App::new(|cx| ui(cx)) // create a new app with the ui function
        .title("Widget Gallery (examples/widget_gallery.rs)") // set the window title
        .style("examples/style/widget-gallery.css") // load a custom stylesheet
        .run(); // run the app
}
