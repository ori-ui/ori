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
    with_state(User::default, |_data, user| {
        let name = text_input().bind_text(|(_, user): &mut (_, User)| &mut user.name);

        let age = hstack![
            text(format!("Age: {}", user.age)),
            button(text("Add")).on_press(move |_, (_, user): &mut (_, User)| { user.age += 1 })
        ];

        let submit = button(text("Submit"))
            .on_press(move |_, (data, user): &mut (Data, User)| {
                data.users.push(user.clone());
                *user = User::default();
            })
            .color(style(Palette::ACCENT));

        vstack![container(vstack![name, age]), submit]
    })
}

fn app(data: &mut Data) -> impl View<Data> {
    let mut users = Vec::new();

    for user in data.users.iter_mut() {
        let fields = hstack![
            text(format!("Name: {},", user.name)),
            text(format!("Age: {}", user.age))
        ]
        .gap(em(1.0));

        let user = container(fields)
            .padding(em(0.5))
            .align(Alignment::CENTER)
            .background(style(Palette::SECONDARY))
            .border_radius(em(0.5));

        users.push(user);
    }

    let users = flex(
        1.0,
        pad(em(1.0), vscroll(vstack(users).align_items(Align::Stretch))),
    );

    center(hstack![form(), users])
}

fn main() {
    App::new(app, Data::default())
        .title("With State (examples/with_state.rs)")
        .run();
}
