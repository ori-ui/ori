# Ori
[![Crates.io](https://img.shields.io/crates/v/ori)](https://crates.io/crates/ori)
[![Documentation](https://img.shields.io/docsrs/ori)](https://docs.rs/ori/latest)

Ori is a reactive ui framework for creating native applications for rust.

```rust
use ori::prelude::*;

// ui is a function that takes a Scope returns a Node
fn ui(cx: Scope) -> Node {
    // create a signal that will hold the state of the counter
    let counter = cx.signal(0);

    // render the ui using the view! macro
    view! {
        <Button on:click=move |_| *counter.modify() += 1>
            "Click me!"
        </Button>
        { format!("Clicked {} times", counter.get()) }
    }
}

fn main() {
    // start the application
    App::new(ui).run();
}
```

## Examples
A [`calculator`](examples/calculator.rs) made with ori.

![Calculator image](assets/calculator.png)

## License
Ori is dual-licensed under either:
 - MIT
 - Apache License, Version 2.0
