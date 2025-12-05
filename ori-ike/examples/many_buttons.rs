use ori_ike::prelude::*;

fn ui(_: &mut ()) -> impl Effect<()> + use<> {
    let mut rows = Vec::new();

    for _ in 0..50 {
        let mut row = Vec::new();

        for _ in 0..50 {
            row.push(button(label("u"), |_| {}));
        }

        rows.push(hstack(row));
    }

    window(vstack(rows)).title("Many buttons")
}

fn main() {
    App::new().run(&mut (), ui);
}
