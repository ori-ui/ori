use ori::prelude::*;

const BORDER_TOP: Key<BorderWidth> = Key::new("todos.border_top");

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

fn input() -> impl View<Data> {
    let input = text_input()
        .placeholder("What needs to be done?")
        .on_submit(|_, data: &mut Data, text| data.input(text))
        .font_size(20.0);

    container(input).padding([em(4.0), em(1.0)]).width(em(28.0))
}

fn todo(index: usize, todo: &mut Todo) -> impl View<Todo> {
    let completed = checkbox(todo.completed, |_, data: &mut Todo| data.toggle());

    let title_color = if todo.completed {
        style(Palette::TEXT_BRIGHTER)
    } else {
        style(Palette::TEXT)
    };

    let title = text(&todo.text).font_size(20.0).color(title_color);

    let remove = button(text("×"), move |cx, _: &mut Todo| {
        // because we don't have access to the Data struct here
        // we send a command to the delegate
        cx.cmd(RemoveTodo(index));
    })
    .fancy(4.0)
    .padding([em(0.4), em(0.1)])
    .color(hsl(353.0, 0.6, 0.72));

    let left = hstack![completed, title].center_items().gap(em(1.5));
    let row = hstack![left, remove]
        .center_items()
        .justify_content(Justify::SpaceBetween);

    container(row)
        .padding(em(1.0))
        .border_width(style(BORDER_TOP))
        .width(em(28.0))
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
        return any(());
    }

    fn color(a: Selection, b: Selection) -> Color {
        if a == b {
            style(Palette::ACCENT)
        } else {
            style(Palette::PRIMARY)
        }
    }

    let all = button(text("All"), |_, data: &mut Data| {
        data.selection = Selection::All
    })
    .fancy(4.0)
    .color(color(data.selection, Selection::All))
    .padding([5.0, 3.0]);

    let active = button(text("Active"), |_, data: &mut Data| {
        data.selection = Selection::Active
    })
    .fancy(4.0)
    .color(color(data.selection, Selection::Active))
    .padding([5.0, 3.0]);

    let completed = button(text("Completed"), |_, data: &mut Data| {
        data.selection = Selection::Completed
    })
    .fancy(4.0)
    .color(color(data.selection, Selection::Completed))
    .padding([5.0, 3.0]);

    let items = hstack![all, active, completed].gap(em(1.0));
    let row = hstack![active_count(data), items]
        .center_items()
        .justify_content(Justify::SpaceBetween);

    any(container(row)
        .width(em(26.0))
        .padding([em(1.0), em(0.5)])
        .border_width(style(BORDER_TOP)))
}

fn app(data: &mut Data) -> impl View<Data> {
    let rows = vstack![input(), todos(data), selection(data)]
        .center_items()
        .gap(0.0);

    align(
        (0.5, 0.2),
        vstack![title(), rows].center_items().gap(em(1.0)),
    )
    .background(style(Palette::BACKGROUND))
}

fn theme() -> Theme {
    Theme::new()
        .with(container::BACKGROUND, Palette::BACKGROUND_DARK)
        .with(container::BORDER_WIDTH, BorderWidth::all(0.0))
        .with(container::BORDER_RADIUS, BorderRadius::all(0.0))
        .with(container::BORDER_COLOR, Palette::SECONDARY_DARK)
        .with(BORDER_TOP, BorderWidth::new(1.0, 0.0, 0.0, 0.0))
        .with(checkbox::BORDER_RADIUS, BorderRadius::all(em(0.75)))
}

// we define a delegate to handle the custom command
fn delegate(cx: &mut DelegateCx, data: &mut Data, event: &Event) -> bool {
    // when we receive the command we remove the todo
    if let Some(remove) = event.get::<RemoveTodo>() {
        data.remove_todo(remove.0);

        cx.request_rebuild();

        return true;
    }

    false
}

fn main() {
    App::new(app, Data::default())
        .title("Todos (examples/todos.rs)")
        .delegate(delegate)
        .theme(theme)
        .run();
}
