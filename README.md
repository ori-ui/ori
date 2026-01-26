# Ori

Ori is a framework for building user interfaces in a declarative manner.

## Example

```rust
use ori_gtk4::prelude::*;

// Here we define our `Data`, the state of our application, this can be anything but in this case
// it's a struct with a `count` field.
struct Data {
    count: u32,
}

// The most important concept in Ori is the `View` trait. A `View` represents the current state of
// your UI based on your `Data`. The `ui` function is called every time the `Data` changes, or more
// accurately is estimated to change. The new `View` is then compared to the previous `View` and
// the differences are applied to the UI.
fn counter(data: &Data) -> impl View<Data> + use<> {
    let text = label(format!("Clicked {} times!", data.count));

    // A button is created, taking a closure mutating our `Data` when the button is clicked. Note
    // that this closure returns a type that can be converted into an `Action`. The default
    // `Action`, i.e. `()` the unit value, is to rebuild the UI by calling the `ui` function. Other
    // actions include sending `Messages` or spawning futures.
    button(text, |data: &mut Data| data.count += 1)
        .halign(Align::Center)
        .valign(Align::Center)
}

// You might notice that `ui` returns an `Effect` rather than a `View`. An `Effect` is simply a
// `View` with no element, meaning it produces no UI and only represents a side-effect. `window` is
// an `Effect` because it *produces* no UI, it only creates a window that contains some UI.
fn ui(data: &mut Data) -> impl Effect<Data> + use<> {
    window(counter(data))
}

// Lastly we simply instantiate our data and run the app with our `ui` function.
fn main() {
    let data = Data { count: 0 };

    App::new().run(data, ui).unwrap();
}
```
