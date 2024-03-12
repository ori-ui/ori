use ori::prelude::*;

// We create our application data struct that holds all the state of our app.
//
// In this case, we only need a counter.
#[derive(Default)]
struct Data {
    counter: u32,
}

// We create a counter button component that increments the `Data::counter` when clicked.
//
// This returns a type that implements the `View` trait with `Data`.
fn counter_button() -> impl View<Data> {
    // We create a button with the text "Click me!", and a fancy value of `pt(4.0)`.
    //
    // `pt` uses the scale fractor from the window to convert points to pixels.
    let counter = button(text("Click me!")).fancy(4.0);

    // We use the `on_click` function to attach a callback to the button that
    // increments the counter.
    //
    // Note that the callback is a closure that takes a mutable reference to
    // an `EventCx` and a mutable reference to the `Data` struct.
    let counter = on_click(counter, |_, data: &mut Data| data.counter += 1);
    tooltip("Click to increment the counter!", counter)
}

// We create our app function that creates the UI of our app.
//
// This will be called every time the UI needs to be rebuilt,
// eg. when the a button is clicked.
fn app(data: &mut Data) -> impl View<Data> {
    // We use the `vstack!` macro to create a vertical stack of views.
    let content = vstack![counter_button(), text!("Clicked {} time(s)", data.counter)];

    // We use the `center` function to center the content in the window.
    center(content)
}

fn main() {
    let window = WindowDescriptor::new().title("Counter (examples/counter.rs)");

    // We create a new app with our `app` function and initial `Data` struct.
    // Then we set the title of the window and run the app.
    Launcher::new(Data::default()).window(window, app).launch();
}
