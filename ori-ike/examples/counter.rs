use ori_ike::prelude::*;

struct Data {
    count: u32,
}

fn counter(data: &mut Data) -> impl View<Data> + use<> {
    center(button(
        label(format!("count {}", data.count)),
        |data: &mut Data| data.count += 1,
    ))
}

fn ui(data: &mut Data) -> impl Effect<Data> + use<> {
    window(counter(data))
}

fn main() {
    let mut data = Data { count: 0 };

    App::new().run(&mut data, ui);
}
