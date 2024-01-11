use std::time::Duration;

use ori::prelude::*;

#[derive(Default)]
struct Data {
    futures_completed: u32,
}

fn app(data: &mut Data) -> impl View<Data> {
    let button = button(text("Click me!")).fancy(4.0);

    let button = on_click(button, |cx, _| {
        let proxy = cx.proxy();

        cx.spawn_async(async move {
            info!("Future started!");

            async_std::task::sleep(Duration::from_secs(1)).await;

            proxy.cmd("Hello from the future!");
        });
    });

    center(vstack![
        button,
        text(format!("Futures completed: {}", data.futures_completed))
    ])
}

fn delegate(cx: &mut DelegateCx, data: &mut Data, event: &Event) {
    if let Some(msg) = event.get::<&str>() {
        info!("Future says: {}", msg);
        data.futures_completed += 1;
        cx.request_rebuild();
    }
}

fn main() {
    let window = WindowDescriptor::new().title("Futures (examples/futures.rs)");

    Launcher::new(Data::default())
        .window(window, app)
        .delegate(delegate)
        .launch();
}
