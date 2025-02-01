use ori::prelude::*;
use ori_font_awesome as fa;

// the selection of the todos
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
#[derive(Debug)]
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
#[derive(Default, Debug)]
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
        .on_submit(|cx, data: &mut Data, text| {
            data.input(text.to_string());
            cx.rebuild();
            cx.focus();
        })
        .font_size(20.0);

    let border = border as i32 as f32 * 2.0;
    let input = container(pad([64.0, 16.0], input)).border_width([0.0, 0.0, border, 0.0]);

    width(28.0 * 16.0, input)
}

fn theme_button(data: &mut Data) -> impl View<Data> {
    let icon = if data.dark_mode {
        fa::icon("moon").color(Theme::light().contrast)
    } else {
        fa::icon("sun").color(Theme::dark().contrast)
    };

    let color = if data.dark_mode {
        Theme::light().background
    } else {
        Theme::dark().background
    };

    let button = button(icon).fancy(4.0).color(color);

    on_click(button, |cx, data: &mut Data| {
        data.dark_mode = !data.dark_mode;
        cx.rebuild();
    })
}

fn todo(index: usize, todo: &mut Todo) -> impl View<Todo> {
    let completed = checkbox(todo.completed).border_radius(12.0);
    let completed = on_click(completed, |cx, data: &mut Todo| {
        data.toggle();
        cx.rebuild();
    });
    let completed = tooltip(completed, "Toggle whether the todo is completed");

    let title_color = if todo.completed {
        Theme::CONTRAST_LOW
    } else {
        Theme::CONTRAST
    };

    let title = text(&todo.text).font_size(20.0).color(title_color);

    let remove = button(fa::icon("xmark"))
        .fancy(4.0)
        .padding(5.0)
        .color(Theme::DANGER);

    let remove = on_click(remove, move |cx, _: &mut Todo| {
        // because we don't have access to the Data struct here
        // we send a command to the delegate
        cx.cmd(RemoveTodo(index));
    });

    let remove = tooltip(remove, "Remove todo");

    let left = hstack![completed, title].gap(24.0);
    let row = hstack![left, remove].justify(Justify::SpaceBetween);

    let container = container(pad(16.0, row));

    if index > 0 {
        width(28.0 * 16.0, container.border_width([0.0, 0.0, 2.0, 0.0]))
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

        let todo = focus(todo(i, item), move |data: &mut Data, lens| {
            lens(&mut data.todos[i])
        });
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

    text(active_text).font_size(16.0)
}

fn selection_button(data: &Data, selection: Selection) -> impl View<Data> {
    let color = if data.selection == selection {
        Theme::ACCENT
    } else {
        Theme::PRIMARY
    };

    let view = button(text!("{:?}", selection).color(Theme::SURFACE))
        .fancy(4.0)
        .color(color)
        .padding([5.0, 3.0]);

    on_click(view, move |cx, data: &mut Data| {
        data.selection = selection;
        cx.rebuild();
    })
}

fn selection(data: &mut Data) -> impl View<Data> {
    if data.todos.is_empty() {
        return None;
    }

    let all = selection_button(data, Selection::All);
    let active = selection_button(data, Selection::Active);
    let completed = selection_button(data, Selection::Completed);

    let items = hstack![all, active, completed].gap(16.0);
    let row = hstack![active_count(data), items].justify(Justify::SpaceBetween);

    let container = container(pad([16.0, 8.0], row)).border_width([2.0, 0.0, 0.0, 0.0]);

    Some(width(26.0 * 16.0, container))
}

fn ui(data: &mut Data) -> impl View<Data> {
    let rows = vstack![
        input(!data.todos.is_empty()),
        expand(vscroll(todos(data))),
        selection(data)
    ]
    .gap(0.0);

    let stack = vstack![title(), expand(rows)].gap(16.0);
    let content = zstack![align((0.5, 0.2), stack), top_right(theme_button(data))];

    let style = if data.dark_mode {
        Theme::dark()
    } else {
        Theme::light()
    };

    let view = background(Theme::BACKGROUND, pad(64.0, content));
    with_styles(style, view)
}

struct Delegate;

impl AppDelegate<Data> for Delegate {
    fn event(&mut self, cx: &mut DelegateCx<Data>, data: &mut Data, event: &Event) -> bool {
        if let Some(&RemoveTodo(index)) = event.cmd() {
            data.remove_todo(index);
            cx.rebuild();

            return true;
        }

        false
    }
}

fn main() {
    ori::log::install().unwrap();

    let window = Window::new().title("Todos (examples/todos.rs)");

    let app = AppBuilder::new().window(window, ui).delegate(Delegate);

    ori::run(app, &mut Data::default()).unwrap();
}
