//! This is the example from the readme.

use ori::prelude::*;

fn ui(cx: Scope) -> Element {
    let counter = cx.signal(0);

    view! {
        <Button on:click=move |_| *counter.modify() += 1>
            "Click me!"
        </Button>
        { format!("Clicked {} times", counter.get()) }
    }
}

fn main() {
    App::new(ui).run();
}
