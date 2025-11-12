# Ori

Ori is a framework for building user interfaces in a declarative manner.

## Example

```rust
use ori_gtk4::prelude::*;

struct Data {
    count: u32,
}


// The most important concept in Ori is the `View` trait. A `View` represents the
// current state of your UI based on your `Data`. The `ui` function is called
// every time the `Data` changes, or more accurately is estimated to change. The
// new `View` is then compared to the previous `View` and the differences are
// applied to the UI.
fn ui(data: &mut Data) -> impl Effect<Data> + use<> {
    window(counter(data))
}

fn counter(data: &Data) -> impl View<Data> + use<> {
    let text = label(format!("clicked {} times", data.count));

    button(text, |data: &mut Data| data.count += 1)
        .halign(Align::Center)
        .valign(Align::Center)
}

fn main() {
    let data = Data { count: 0 };

    App::new().run(data, ui).unwrap();
}
```
