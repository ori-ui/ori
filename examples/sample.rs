use ily::prelude::*;

fn ui<'a>(cx: Scope<'a>) -> impl View {
    let counter = cx.signal(1);

    view! {
        <Div>
            <!-- "Counter" -->
            <Button on:press=|_| *counter.modify() += 1>
                <Text text=format!("Counter: {}", counter.get()) />
            </Button>
        </Div>
    }
}

fn main() {
    App::new(|cx| ui(cx)).run();
}
