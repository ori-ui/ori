use ori_ike::prelude::*;

fn ui(_: &mut ()) -> impl Effect<()> + use<> {
    window(picture(
        Fit::Cover,
        include_svg!("Ghostscript_Tiger.svg"),
    ))
}

fn main() {
    App::new().run(&mut (), ui);
}
