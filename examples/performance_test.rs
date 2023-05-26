use ori::prelude::*;

fn ui(_cx: Scope) -> Element {
    let mut div = Div::new().class("row");

    for _ in 0..40 {
        let mut child = Div::new().class("column");

        for _ in 0..40 {
            child.add_child(Button::new(Text::new("A")));
        }

        div.add_child(child);
    }

    Element::new(div)
}

fn main() {
    App::new(ui)
        .style("examples/style/performance-test.css")
        .run();
}
