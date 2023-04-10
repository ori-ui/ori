use ily::*;

fn ui<'a>(cx: Scope<'a>, color: &'a Signal<Color>) -> impl View {
    Div::new()
        .child(cx, move |cx| {
            Button::new().on_press(cx, move |_| {
                color.set(Color::CYAN);
            })
        })
        .child(cx, move |cx| {
            Button::new().on_press(cx, move |_| {
                color.set(Color::WHITE);
            })
        })
        .padding(10.0)
        .gap(10.0)
        .background(*color.get())
}

fn main() {
    App::new(|cx| {
        let color = cx.signal(Color::WHITE);
        cx.dynamic(|cx| ui(cx, color))
    })
    .run();
}
