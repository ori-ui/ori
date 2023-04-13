use ily::prelude::*;

fn ui<'a>(cx: Scope<'a>) -> impl View {
    let counter = cx.signal(1);
    let checked = cx.signal(false);

    view! {
        <Div class=if *checked.get() { "column" } else { "row" }>
            <Button on:press=|_| *counter.modify() += 1>
                <Text text=format!("Counter: {}", counter.get()) />
            </Button>
            <Checkbox bind:checked=checked />
        </Div>
    }
}

fn main() {
    App::new(|cx| ui(cx)) // create a new app with the ui function
        .title("Widget Gallery (examples/widget_gallery.rs)") // set the window title
        .style("examples/widget_gallery.css") // load a custom stylesheet
        .run(); // run the app
}
