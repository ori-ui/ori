use ori::prelude::*;

#[derive(Default)]
struct Data {
    windows: Vec<WindowId>,
}

fn window(_data: &mut Data) -> impl View<Data> {
    center(text("Hello World!"))
}

fn open_window_button() -> impl View<Data> {
    let open_window = button(text("Open window"))
        .fancy(pt(4.0))
        .color(style(Palette::ACCENT));

    on_click(open_window, |cx, data: &mut Data| {
        let desc = WindowDescriptor::new()
            .title("Multi Window Popup")
            .size(300, 300);

        data.windows.push(desc.id);

        info!("Window {} opened", desc.id);

        cx.open_window(desc, window);
    })
}

fn close_window_button(data: &mut Data) -> impl View<Data> {
    let close_window = transition(ease(0.5), data.windows.is_empty(), |_cx, t| {
        let active = style(Palette::PRIMARY);
        let inactive = style(Palette::SECONDARY_DARKER);

        button(text("Close window"))
            .color(active.mix(inactive, t))
            .fancy(pt(4.0))
    });

    on_click(close_window, |cx, data: &mut Data| {
        if let Some(window) = data.windows.pop() {
            cx.close_window(window);

            info!("Window {} closed", window);
        }
    })
}

fn app(data: &mut Data) -> impl View<Data> {
    let stack = vstack![open_window_button(), close_window_button(data)].gap(rem(0.5));

    center(stack)
}

struct AppDelegate;

impl Delegate<Data> for AppDelegate {
    fn event(&mut self, cx: &mut DelegateCx, data: &mut Data, event: &Event) {
        if let Some(request) = event.get::<CloseRequested>() {
            data.windows.retain(|window| *window != request.window);
            cx.request_rebuild();

            info!("Window {} closed", request.window);
        }
    }
}

fn main() {
    App::new(app, Data::default())
        .title("Multi Window (examples/multi_window.rs)")
        .delegate(AppDelegate)
        .run();
}
