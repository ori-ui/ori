use ori_ike::prelude::*;

struct Data {
    todos: Vec<Todo>,
}

impl Data {
    fn add_todo(&mut self, name: String) {
        self.todos.push(Todo::new(name));
    }
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

    button(
        size([16.0, 16.0], checkmark),
        move |data: &mut Data| {
            data.todos[i].done = !data.todos[i].done;
        },
    )
    .padding(Padding::all(2.0))
}

fn todo_remove(i: usize, _todo: &Todo) -> impl View<Data> + use<> {
    let xmark = using_or_default(|_, palette: &Palette| {
        picture(Fit::Contain, include_svg!("xmark.svg")).color(palette.contrast)
    });

    button(
        size([16.0, 16.0], xmark),
        move |data: &mut Data| {
            data.todos.remove(i);
        },
    )
    .padding(Padding::all(2.0))
}

fn todo(i: usize, todo: &Todo) -> impl View<Data> + use<> {
    container(
        hstack((
            todo_done(i, todo),
            todo_remove(i, todo),
            center(label(&todo.name)),
        ))
        .gap(12.0),
    )
}

fn todos(data: &mut Data) -> impl View<Data> + use<> {
    vstack(
        data.todos
            .iter()
            .enumerate()
            .rev()
            .map(|(i, t)| todo(i, t))
            .collect::<Vec<_>>(),
    )
}

fn ui(data: &mut Data) -> impl Effect<Data> + use<> {
    window(center(width(
        200.0,
        vstack((name_entry(), todos(data))),
    )))
}

fn main() {
    let mut data = Data { todos: Vec::new() };

    App::new().run(&mut data, ui);
}
