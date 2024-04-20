use ori::prelude::*;

struct OpenWindow;
struct CloseWindow(WindowId);

#[derive(Default)]
struct Data {
    windows: Vec<WindowId>,
}

fn window(_data: &mut Data) -> impl View<Data> {
    let close = button(fa::icon("xmark"))
        .color(hsl(0.0, 0.5, 0.5))
        .border_radius(0.0);

    let close = top_right(on_click(close, |cx, data: &mut Data| {
        let window_id = cx.window().id();
        cx.cmd(CloseWindow(window_id));

        data.windows.retain(|window| *window != window_id);
    }));

    let content = container(zstack![center(text("Hello World!")), close])
        .border_radius([16.0, 0.0, 16.0, 0.0])
        .border_width(2.0);

    on_press(trigger(content), |cx, _| {
        cx.window().drag();
    })
}

fn open_window_button() -> impl View<Data> {
    let open_window = button(text("Open window"))
        .fancy(4.0)
        .color(palette().accent);

    on_click(open_window, |cx, _: &mut Data| {
        cx.cmd(OpenWindow);
    })
}

fn close_window_button(data: &mut Data) -> impl View<Data> {
    let close_window = transition(ease(0.5), !data.windows.is_empty(), |_cx, _, t| {
        let active = palette().primary;
        let inactive = palette().surface_secondary;

        button(text("Close window"))
            .color(inactive.mix(active, t))
            .fancy(4.0)
    });

    on_click(close_window, |cx, data: &mut Data| {
        if let Some(window) = data.windows.pop() {
            cx.cmd(CloseWindow(window));

            info!("Window {} closed", window);
        }
    })
}

fn app(data: &mut Data) -> impl View<Data> {
    let stack = vstack![open_window_button(), close_window_button(data)].gap(8.0);

    center(stack)
}

struct AppDelegate;

impl Delegate<Data> for AppDelegate {
    fn event(&mut self, cx: &mut DelegateCx<Data>, data: &mut Data, event: &Event) -> bool {
        if let Some(request) = event.cmd::<CloseRequested>() {
            data.windows.retain(|window| *window != request.window);
            cx.request_rebuild();

            info!("Window {} closed", request.window);
        }

        if event.is_cmd::<OpenWindow>() {
            let desc = WindowDescriptor::new()
                .title("Multi Window Popup")
                .resizable(false)
                .decorated(false)
                .color(Color::TRANSPARENT)
                .size(300, 300);

            data.windows.push(desc.id);

            info!("Window {} opened", desc.id);

            cx.open_window(desc, window);
            cx.request_rebuild();
        }

        if let Some(CloseWindow(window)) = event.cmd() {
            cx.close_window(*window);
        }

        false
    }
}

fn main() {
    let window = WindowDescriptor::new().title("Multi Window (examples/multi_window.rs)");

    let app = App::build().window(window, app).delegate(AppDelegate);

    ori::launch(app, Data::default()).unwrap();
}
