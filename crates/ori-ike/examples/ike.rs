use ori::views::effects;
use ori_ike::{App, Effect};

struct Data {
    count: u32,
}

fn ui(data: &mut Data) -> impl Effect<Data> + use<> {
    window(())
}

fn main() {
    let mut data = Data { count: 0 };

    App::new().run(&mut data, ui);
}
