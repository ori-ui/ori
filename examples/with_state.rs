use ori::prelude::*;

#[derive(Clone, Default)]
struct User {
    name: String,
    age: u8,
}

#[derive(Default)]
struct Data {
    users: Vec<User>,
}

fn form() -> impl View<Data> {
    // the `with_state` view is used to store state that is not part of the data
    with_state(User::default, |_, _| {
        build(|cx, (user, _): &mut (User, _)| {
            let name = text_input()
                .text(&user.name)
                .on_input(|_, (user, _): &mut (User, _), text| user.name = text);

            let age = hstack![
                text!("Age: {}", user.age),
                on_click(button(text("Add")), move |cx, (user, _): &mut (User, _)| {
                    user.age += 1;
                    cx.rebuild();
                })
            ];

            let submit = button(text("Submit")).color(cx.theme().accent);

            let submit = on_click(submit, |cx, (user, data): &mut (User, Data)| {
                data.users.push(user.clone());
                *user = User::default();
                cx.rebuild();
            });

            vstack![vstack![name, age], submit]
        })
    })
}

fn ui() -> impl View<Data> {
    build(|cx, data: &mut Data| {
        let mut users = Vec::new();

        for user in data.users.iter_mut() {
            let fields =
                hstack![text!("Name: {},", user.name), text!("Age: {}", user.age)].gap(16.0);

            let user = container(pad(16.0, fields))
                .background(cx.theme().surface)
                .border_radius(8.0);

            users.push(center(user));
        }

        let users = pad(16.0, vscroll(vstack(users)));

        center(hstack![form(), users])
    })
}

fn main() {
    ori::log::install().unwrap();

    let window = Window::new().title("With State (examples/with_state.rs)");

    let app = App::build().window(window, ui);

    ori::run(app, &mut Data::default()).unwrap();
}
