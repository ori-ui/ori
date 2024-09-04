use std::time::Duration;

use ori::prelude::*;

#[derive(Default)]
struct Data {
    futures_completed: u32,
}

fn ui(data: &mut Data) -> impl View<Data> {
    let button = button(text("Click me!")).fancy(4.0);

    let button = on_click(button, |cx, _| {
        cx.cmd_async(async move {
            info!("Future started!");

            async_std::task::sleep(Duration::from_secs(1)).await;

            "Hello from the future!"
        });
    });

    center(vstack![
        button,
        text!("Futures completed: {}", data.futures_completed)
    ])
}

struct AppDelegate;

impl Delegate<Data> for AppDelegate {
    fn event(&mut self, cx: &mut DelegateCx<Data>, data: &mut Data, event: &Event) -> bool {
        if let Some(msg) = event.cmd::<&str>() {
            info!("Future says: {}", msg);
            data.futures_completed += 1;
            cx.rebuild();
        }

        false
    }
}

fn main() {
    ori::log::install().unwrap();

    let window = Window::new().title("Futures (examples/futures.rs)");

    let app = App::build().window(window, ui).delegate(AppDelegate);

    ori::run(app, &mut Data::default()).unwrap();
}
