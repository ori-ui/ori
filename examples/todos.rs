use ori::prelude::*;

// the selection of the todos
#[derive(Clone, Copy, Default, PartialEq)]
enum Selection {
    // show all todos
    #[default]
    All,
    // show active todos
    Active,
    // show completed todos
    Completed,
}

// here we define a custom command we can send to the delegate
struct RemoveTodo(usize);

// a todo
struct Todo {
    text: String,
    completed: bool,
}

impl Todo {
    fn new(text: impl ToString) -> Todo {
        Todo {
            text: text.to_string(),
            completed: false,
        }
    }

    fn toggle(&mut self) {
        self.completed = !self.completed;
    }
}

// data for the app
#[derive(Default)]
struct Data {
    todos: Vec<Todo>,
    selection: Selection,
    dark_mode: bool,
}

impl Data {
    fn input(&mut self, title: String) {
        info!("Added todo '{}'", title);

        let todo = Todo::new(title);
        self.todos.push(todo);
    }

    fn remove_todo(&mut self, index: usize) {
        self.todos.remove(index);
        info!("Removed todo #{}", index);
    }
}

fn title() -> impl View<Data> {
    text("todos").font_size(50.0)
}

fn input(border: bool) -> impl View<Data> {
    let input = text_input()
        .placeholder("What needs to be done?")
        .on_submit(|_, data: &mut Data, text| data.input(text.to_string()))
        .font_size(20.0);

    let border = border as i32 as f32 * 2.0;
    let input = container(pad([64.0, 16.0], input)).border_bottom(border);

    width(28.0 * 16.0, input)
}

fn theme_button(data: &mut Data) -> impl View<Data> {
    let icon = if data.dark_mode {
        fa::icon("moon").color(Palette::light().text())
    } else {
        fa::icon("sun").color(Palette::dark().text())
    };

    let color = if data.dark_mode {
        Palette::light().background()
    } else {
        Palette::dark().background()
    };

    let button = button(icon).fancy(4.0).color(color);

    on_click(button, |_, data: &mut Data| {
        data.dark_mode = !data.dark_mode;
    })
}

fn todo(index: usize, todo: &mut Todo) -> impl View<Todo> {
    let completed = checkbox(todo.completed).border_radius(12.0);
    let completed = on_press(completed, |_, data: &mut Todo| data.toggle());
    let completed = tooltip("Toggle whether the todo is completed", completed);

    let title_color = if todo.completed {
        palette().text_lighter()
    } else {
        palette().text()
    };

    let title = text(&todo.text).font_size(20.0).color(title_color);

    let remove = button(fa::icon("xmark"))
        .fancy(4.0)
        .padding(5.0)
        .color(hsl(353.0, 0.6, 0.72));

    let remove = on_click(remove, move |cx, _: &mut Todo| {
        // because we don't have access to the Data struct here
        // we send a command to the delegate
        cx.cmd(RemoveTodo(index));
    });

    let remove = tooltip("Remove todo", remove);

    let left = hstack![completed, title].gap(24.0);
    let row = hstack![left, remove].justify(Justify::SpaceBetween);

    let container = container(pad(16.0, row));

    if index > 0 {
        width(28.0 * 16.0, container.border_bottom(2.0))
    } else {
        width(28.0 * 16.0, container)
    }
}

fn todos(data: &mut Data) -> impl View<Data> {
    let mut todos = Vec::new();

    for (i, item) in data.todos.iter_mut().enumerate().rev() {
        match data.selection {
            Selection::Active if item.completed => continue,
            Selection::Completed if !item.completed => continue,
            _ => {}
        }

        let todo = focus(
            move |data: &mut Data, lens| lens(&mut data.todos[i]),
            todo(i, item),
        );
        todos.push(todo);
    }

    vstack(todos).gap(0.0)
}

fn active_count(data: &mut Data) -> impl View<Data> {
    let active = data.todos.iter().filter(|t| !t.completed).count();

    let active_text = if active == 1 {
        String::from("1 item left")
    } else {
        format!("{} items left", active)
    };

    text(active_text).font_size(14.0)
}

fn selection(data: &mut Data) -> impl View<Data> {
    if data.todos.is_empty() {
        return None;
    }

    fn color(a: Selection, b: Selection) -> Color {
        if a == b {
            palette().accent()
        } else {
            palette().primary()
        }
    }

    let all = button(text("All"))
        .fancy(4.0)
        .color(color(data.selection, Selection::All))
        .padding([5.0, 3.0]);
    let active = button(text("Active"))
        .fancy(4.0)
        .color(color(data.selection, Selection::Active))
        .padding([5.0, 3.0]);
    let completed = button(text("Completed"))
        .fancy(4.0)
        .color(color(data.selection, Selection::Completed))
        .padding([5.0, 3.0]);

    let all = on_click(all, |_, data: &mut Data| data.selection = Selection::All);
    let active = on_click(active, |_, data: &mut Data| {
        data.selection = Selection::Active
    });
    let completed = on_click(completed, |_, data: &mut Data| {
        data.selection = Selection::Completed
    });

    let items = hstack![all, active, completed].gap(16.0);
    let row = hstack![active_count(data), items].justify(Justify::SpaceBetween);

    let container = container(pad([16.0, 8.0], row)).border_top(2.0);

    Some(width(26.0 * 16.0, container))
}

fn app(data: &mut Data) -> impl View<Data> {
    let style = if data.dark_mode {
        Palette::dark()
    } else {
        Palette::light()
    };

    styled(style, || {
        let rows = vstack![
            input(!data.todos.is_empty()),
            flex(vscroll(todos(data))),
            selection(data)
        ]
        .gap(0.0);

        let stack = vstack![title(), flex(rows)].gap(16.0);
        let content = zstack![align((0.5, 0.2), stack), top_right(theme_button(data))];

        background(palette().background(), pad(64.0, content))
    })
}

struct AppDelegate;

impl Delegate<Data> for AppDelegate {
    fn event(&mut self, cx: &mut DelegateCx<Data>, data: &mut Data, event: &Event) {
        if let Some(remove) = event.get::<RemoveTodo>() {
            data.remove_todo(remove.0);

            cx.request_rebuild();
            event.handle();
        }
    }
}

fn main() {
    let window = WindowDescriptor::new().title("Todos (examples/todos.rs)");

    Launcher::new(Data::default())
        .window(window, app)
        .delegate(AppDelegate)
        .launch();
}
