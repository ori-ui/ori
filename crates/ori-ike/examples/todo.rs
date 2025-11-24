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
}

impl Todo {
    fn new(name: String) -> Self {
        Self { name }
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

fn todo(i: usize, todo: &Todo) -> impl View<Data> + use<> {
    container(
        hstack((
            button(label("x"), move |data: &mut Data| {
                data.todos.remove(i);
            }),
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
