use ori_ike::{
    App, Effect,
    views::{button, center, label, text_area, vstack, window},
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
        text_area(),
    ))))
}

fn main() {
    let mut data = Data { count: 0 };

    App::new().run(&mut data, ui);
}
