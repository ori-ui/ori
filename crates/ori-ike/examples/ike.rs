use ori::views::any;
use ori_ike::{
    App, Effect,
    views::{button, center, hstack, label, vstack, window},
};

struct Data {
    count: u32,
}

fn ui(data: &mut Data) -> impl Effect<Data> + use<> {
    window(center(vstack((
        button(
            label(format!("count {}", data.count)),
            |data: &mut Data| data.count += 1,
        ),
        if data.count.is_multiple_of(2) {
            any(hstack((label("wahoo"), label("another"))).gap(10.0))
        } else {
            any(label("wahoo"))
        },
    ))))
}

fn main() {
    let mut data = Data { count: 0 };

    App::new().run(&mut data, ui);
}
