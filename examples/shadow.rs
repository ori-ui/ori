use ori::prelude::*;

struct Data;

impl Data {
    pub fn ui(&mut self) -> impl View<Self> {
        let container = container(())
            .background(Color::WHITE)
            .shadow_color(Color::BLACK)
            .shadow_blur(32.0);

        center(size(Size::all(10.0 * 16.0), container))
    }
}

fn main() {
    let window = Window::new().title("Shadow (examples/shadow.rs)");

    let app = App::build().window(window, Data::ui);

    ori::launch(app, Data).unwrap();
}
