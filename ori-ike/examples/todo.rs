use ori_ike::prelude::*;

struct Data {
    todos:  Vec<Todo>,
    filter: Filter,
}

impl Data {
    fn add_todo(&mut self, name: String) {
        self.todos.push(Todo::new(name));
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Filter {
    Done,
    Pending,
    All,
}

struct Todo {
    name: String,
    done: bool,
}

impl Todo {
    fn new(name: String) -> Self {
        Self { name, done: false }
    }
}

fn name_entry() -> impl View<Data> + use<> {
    entry()
        .placeholder("What do you want to do?")
        .on_submit(Data::add_todo)
        .submit_behaviour(SubmitBehaviour {
            keep_focus: true,
            clear_text: true,
        })
        .corner_radius(0.0)
}

fn todo_done(i: usize, _todo: &Todo) -> impl View<Data> + use<> {
    let checkmark = using_or_default(
        move |data: &mut Data, palette: &Palette| {
            let color = if data.todos[i].done {
                palette.success
            } else {
                Color::TRANSPARENT
            };

            picture(Fit::Contain, include_svg!("check.svg")).color(color)
        },
    );

    let button = button(checkmark, move |data: &mut Data| {
        data.todos[i].done = !data.todos[i].done;
    })
    .padding(2.0)
    .corner_radius(12.0);

    size([24.0, 24.0], button)
}

fn todo_remove(i: usize, _todo: &Todo) -> impl View<Data> + use<> {
    let xmark = using_or_default(|_, palette: &Palette| {
        picture(Fit::Contain, include_svg!("xmark.svg")).color(palette.danger)
    });

    let button = button(xmark, move |data: &mut Data| {
        data.todos.remove(i);
    })
    .padding(2.0);

    size([24.0, 24.0], button)
}

fn todo(i: usize, todo: &Todo) -> impl View<Data> + use<> {
    container(
        hstack((
            hstack((
                todo_done(i, todo),
                center(label(&todo.name)),
            ))
            .gap(12.0),
            todo_remove(i, todo),
        ))
        .justify(Justify::SpaceBetween)
        .gap(12.0),
    )
    .border_width([1.0, 0.0, 1.0, 1.0])
    .corner_radius(0.0)
}

fn todos(data: &mut Data) -> impl View<Data> + use<> {
    vstack(
        data.todos
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, t)| match data.filter {
                Filter::Done => t.done,
                Filter::Pending => !t.done,
                Filter::All => true,
            })
            .map(|(i, t)| todo(i, t))
            .collect::<Vec<_>>(),
    )
}

fn filter(kind: Filter, name: &'static str) -> Flex<impl View<Data> + use<>> {
    expand(
        1.0,
        using_or_default(
            move |data: &mut Data, palette: &Palette| {
                button(
                    center(
                        label(name).color(if data.filter == kind {
                            palette.success
                        } else {
                            palette.contrast
                        }),
                    ),
                    move |data: &mut Data| {
                        data.filter = kind;
                    },
                )
                .color(palette.surface(0))
                .border_width([0.0, 0.0, 1.0, 0.0])
                .corner_radius(0.0)
            },
        ),
    )
}

fn filters() -> impl View<Data> + use<> {
    container(
        hstack((
            filter(Filter::Done, "done"),
            filter(Filter::Pending, "pending"),
            filter(Filter::All, "all"),
        ))
        .justify(Justify::SpaceAround),
    )
    .padding(0.0)
    .border_width([1.0, 0.0, 0.0, 1.0])
    .corner_radius(0.0)
}

fn ui(data: &mut Data) -> impl Effect<Data> + use<> {
    provide(
        |_| Palette::paper(),
        window(center(width(
            300.0,
            vstack((
                name_entry(),
                todos(data),
                width(240.0, filters()),
            )),
        ))),
    )
}

fn main() {
    let mut data = Data {
        todos:  Vec::new(),
        filter: Filter::All,
    };

    App::new().run(&mut data, ui);
}
