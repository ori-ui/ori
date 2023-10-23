use ori::prelude::*;

struct Data;

impl Data {
    pub fn ui(&mut self) -> impl View<Self> {
        let container = container(())
            .background(Color::WHITE)
            .shadow_color(Color::BLACK)
            .shadow_blur(rem(2.0));

        center(size(Size::all(rem(10.0)), container))
    }
}

fn main() {
    Launcher::new(Data::ui, Data)
        .title("Shadow (examples/shadow.rs)")
        .launch();
}
