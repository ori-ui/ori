use ily::prelude::*;

fn ui<'a>(cx: Scope<'a>) -> impl View {
    let counter = cx.signal(1);
    let checked = cx.signal(false);

    view! {
        <Div>
            <!-- "Counter" -->
            <Button on:press=|_| *counter.modify() += 1>
                <Text text=format!("Counter: {}", counter.get()) />
            </Button>
            <Checkbox bind:checked=checked />
        </Div>
    }
}

fn main() {
    App::new(|cx| ui(cx)).style("examples/sample.css").run();
}
