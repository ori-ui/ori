use ori::prelude::*;

#[derive(Default)]
struct Data {}

fn app(_data: &mut Data) -> impl View<Data> {
    let click_me = animate(|t: &mut f32, cx, _data, event| {
        if event.is::<HotChanged>() {
            cx.request_animation_frame();
        }

        if let Some(AnimationFrame(dt)) = event.get() {
            if ease(0.2).step(t, cx.is_hot(), *dt) {
                cx.request_animation_frame();
            }
        }

        let border = Color::RED.mix(Color::GREEN, *t);

        button(text("Click me"))
            .border_width(pt(2.0))
            .border_color(border)
            .fancy(pt(4.0))
    });

    center(on_click(click_me, |cx, _| info!("Clicked!")))
}

fn main() {
    App::new(app, Data::default())
        .title("Animate (examples/animate.rs)")
        .run();
}
