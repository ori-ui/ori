use ori::views::effects;
use ori_ike::{App, Effect};

struct Data {
    count: u32,
}

fn ui(data: &mut Data) -> impl Effect<Data> + use<> {
    effects(())
}

fn main() {
    let mut data = Data { count: 0 };

    App::new().run(&mut data, ui);
}
