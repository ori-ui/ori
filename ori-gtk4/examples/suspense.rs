use std::time::Duration;

use futures_timer::Delay;
use ori_gtk4::prelude::*;

fn ui(_: &mut ()) -> impl Effect<()> + use<> {
    let suspense = freeze(|| {
        suspense(
            label("Waiting for future to complete..."),
            async {
                Delay::new(Duration::from_secs(5)).await;

                label("Future has now completed!")
            },
        )
    });

    window(suspense.halign(Align::Center).valign(Align::Center))
}

fn main() {
    App::new().run((), ui).unwrap();
}
