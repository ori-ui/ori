use ori::prelude::*;
use ori_font_awesome as fa;

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
        cx.rebuild();
    }));

    let content = container(zstack![center(text("Hello World!")), close])
        .border_radius([16.0, 0.0, 16.0, 0.0])
        .border_width(2.0);

    on_press(trigger(content), |_, _| {})
}

fn open_window_button() -> impl View<Data> {
    let open_window = button(text("Open window")).color(Theme::ACCENT).fancy(4.0);

    on_click(open_window, |cx, _: &mut Data| {
        cx.cmd(OpenWindow);
    })
}

fn close_window_button() -> impl View<Data> {
    let close_window = button(text("Close window")).color(Theme::DANGER).fancy(4.0);

    on_click(close_window, |cx, data: &mut Data| {
        if let Some(window) = data.windows.pop() {
            cx.cmd(CloseWindow(window));
            cx.rebuild();

            info!("Window {} closed", window);
        }
    })
}

fn ui(_data: &mut Data) -> impl View<Data> {
    let stack = vstack![open_window_button(), close_window_button()].gap(8.0);

    center(stack)
}

struct Delegate;

impl AppDelegate<Data> for Delegate {
    fn event(&mut self, cx: &mut DelegateCx<Data>, data: &mut Data, event: &Event) -> bool {
        if let Some(request) = event.cmd::<CloseRequested>() {
            data.windows.retain(|w| *w != request.window);
            cx.rebuild();

            info!("Window {} closed", request.window);
        }

        if let Event::CloseRequested(e) = event {
            data.windows.retain(|w| *w != e.window);
            cx.rebuild();

            info!("Window {} closed", e.window);
        }

        if event.is_cmd::<OpenWindow>() {
            let desc = Window::new()
                .title("Multi Window Popup")
                .resizable(false)
                .decorated(false)
                .color(Some(Color::TRANSPARENT))
                .size(300, 300);

            data.windows.push(desc.id());

            info!("Window {} opened", desc.id());

            cx.open_window(desc, window);
            cx.rebuild();
        }

        if let Some(CloseWindow(window)) = event.cmd() {
            cx.close_window(*window);
        }

        false
    }
}

fn main() {
    ori::log::install().unwrap();

    let window = Window::new().title("Multi Window (examples/multi_window.rs)");

    let app = App::build().window(window, ui).delegate(Delegate);

    ori::run(app, &mut Data::default()).unwrap();
}
