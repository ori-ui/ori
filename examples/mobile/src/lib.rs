use ori::prelude::*;

fn ui() -> impl View {
    let view = button(text!("Hello, mobile!").font_size(24.0)).fancy(8.0);
    let view = on_click(view, |_, _| {
        println!("Hello, world!");
    });

    let text_input = text_input().font_size(24.0).placeholder("Type here...");

    center(vstack![view, text_input])
}

#[ori::main]
pub fn main() {
    let window = Window::new().title("Mobile Example");
    ori::run_simple(window, ui).unwrap();
}
