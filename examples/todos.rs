use ori::prelude::*;

const BORDER_TOP: Key<BorderWidth> = Key::new("todos.border_top");

#[derive(Clone, Copy, Default, PartialEq)]
enum Selection {
    #[default]
    All,
    Active,
    Completed,
}

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

#[derive(Default)]
struct Data {
    todos: Vec<Todo>,
    text: String,
    selection: Selection,
}

impl Data {
    fn input(&mut self) {
        let todo = Todo::new(self.text.clone());
        self.todos.insert(0, todo);
        self.text = String::new();
    }
}

fn title() -> impl View<Data> {
    text("todos").font_size(50.0)
}

fn input() -> impl View<Data> {
    let input = text_input(|data: &mut Data| &mut data.text)
        .on_submit(Data::input)
        .placeholder("What needs to be done?")
        .font_size(20.0);

    container(input).padding((em(4.0), em(1.0))).width(em(28.0))
}

fn todo(todo: &mut Todo) -> impl View<Todo> {
    let completed = checkbox(todo.completed, Todo::toggle);

    let title_color = if todo.completed {
        style(Palette::TEXT_BRIGHTER)
    } else {
        style(Palette::TEXT)
    };

    let title = text(&todo.text).font_size(20.0).color(title_color);

    let left = hstack![completed, title].center_items().gap(em(1.5));

    container(left)
        .padding(em(1.0))
        .border_width(style(BORDER_TOP))
        .width(em(28.0))
}

fn todos(data: &mut Data) -> impl View<Data> {
    let mut todos = Vec::new();

    for (i, item) in data.todos.iter_mut().enumerate() {
        match data.selection {
            Selection::Active if item.completed => continue,
            Selection::Completed if !item.completed => continue,
            _ => {}
        }

        let todo = focus(move |data: &mut Data| &mut data.todos[i], todo(item));
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

    Text::new(active_text).font_size(14.0)
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

    let all = button(text("All"), |data: &mut Data| {
        data.selection = Selection::All
    })
    .fancy(4.0)
    .color(color(data.selection, Selection::All))
    .padding((5.0, 3.0));

    let active = button(text("Active"), |data: &mut Data| {
        data.selection = Selection::Active
    })
    .fancy(4.0)
    .color(color(data.selection, Selection::Active))
    .padding((5.0, 3.0));

    let completed = button(text("Completed"), |data: &mut Data| {
        data.selection = Selection::Completed
    })
    .fancy(4.0)
    .color(color(data.selection, Selection::Completed))
    .padding((5.0, 3.0));

    let items = hstack![active_count(data), all, active, completed]
        .justify_content(Justify::SpaceAround)
        .center_items()
        .gap(em(1.0));

    any(container(items)
        .width(em(26.0))
        .padding(em(0.5))
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
        .with(checkbox::BORDER_RADIUS, BorderRadius::all(12.0))
}

fn main() {
    App::new(app, Data::default())
        .title("Todos (examples/todos.rs)")
        .theme(theme())
        .run();
}
