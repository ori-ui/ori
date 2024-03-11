use std::time::Duration;

use crate::{
    canvas::{Background, BorderRadius, BorderWidth, Color, Fragment, Primitive, Quad},
    layout::{Affine, Align, Justify, Point, Rect, FILL},
    text::FontFamily,
    theme::{style, window_size, Palette},
    view::{any, View},
    views::{
        button, collapsing, container, expand, height, hstack, left, on_click, on_draw, pad,
        pad_left, pad_top, size, text, top_left, trigger, vscroll, vstack, vstack_any, width,
        with_state, Button,
    },
};

use super::{DebugData, DebugTree, DebugView};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DebugTab {
    #[default]
    Inspector,
    Profiler,
}

#[derive(Default)]
struct TreeState {
    expanded: bool,
    transform: Affine,
    content: Vec<TreeState>,
}

impl TreeState {
    fn get_mut(&mut self, path: &[usize]) -> &mut Self {
        let mut state = self;

        for &index in path {
            state = &mut state.content[index];
        }

        state
    }
}

#[derive(Default)]
struct DebugState {
    tab: DebugTab,
    tree: TreeState,
    selected_tree: Option<Vec<usize>>,
}

macro_rules! debug_text {
    ($($arg:tt)*) => {
        text!($($arg)*).font_size(12.0).font_family(FontFamily::Monospace)
    };
}

fn debug_tree_expand_button(
    tree: &DebugTree,
    state: &TreeState,
    path: &[usize],
) -> impl View<(DebugData, DebugState)> {
    if tree.content().count() == 0 {
        return Err(width(12.0, ()));
    }

    let symbol = if state.expanded { "-" } else { "+" };
    let symbol = debug_text!("{}", symbol);

    let path = path.to_owned();
    let expand = on_click(
        trigger(symbol),
        move |_, (_, state): &mut (_, DebugState)| {
            let tree_state = state.tree.get_mut(&path);

            tree_state.expanded = !tree_state.expanded;
        },
    );

    Ok(width(12.0, expand))
}

fn debug_tree_name(tree: &DebugTree) -> impl View<(DebugData, DebugState)> {
    debug_text!("{}", tree.short_name())
}

fn debug_tree_size(tree: &DebugTree) -> impl View<(DebugData, DebugState)> {
    debug_text!("[{} x {}]", tree.rect().width(), tree.rect().height())
}

fn debug_tree_child_count(tree: &DebugTree) -> impl View<(DebugData, DebugState)> {
    let count = tree.content().count();

    if count == 0 {
        return None;
    }

    let count = debug_text!("({})", count);

    Some(count)
}

fn debug_tree_hightlight(
    tree: &DebugTree,
    state: &TreeState,
    content: impl View<(DebugData, DebugState)>,
) -> impl View<(DebugData, DebugState)> {
    let rect = tree.rect();
    let transform = state.transform;

    on_draw(trigger(content), move |cx, _, canvas| {
        if cx.is_hot() {
            canvas.draw_fragment(Fragment {
                primitive: Primitive::Quad(Quad {
                    rect,
                    background: Background::new(Color::TRANSPARENT),
                    border_radius: BorderRadius::all(0.0),
                    border_width: BorderWidth::all(2.0),
                    border_color: style(Palette::ACCENT),
                }),
                transform,
                depth: f32::MAX,
                clip: Rect::min_size(Point::ZERO, cx.window().size()),
                view: None,
                pixel_perfect: true,
            });
        }
    })
}

fn debug_tree_header(
    tree: &DebugTree,
    state: &mut TreeState,
    selected: Option<&[usize]>,
    path: &[usize],
) -> impl View<(DebugData, DebugState)> {
    let expand = debug_tree_expand_button(tree, state, path);
    let name = debug_tree_name(tree);
    let size = debug_tree_size(tree);
    let child_count = debug_tree_child_count(tree);

    let stack = hstack![expand, name, size, child_count]
        .align_items(Align::Start)
        .gap(4.0);

    let color = if Some(path) == selected {
        style(Palette::PRIMARY_LIGHT)
    } else {
        style(Palette::SECONDARY)
    };

    let mut header = button(pad_left(path.len() as f32 * 10.0 + 20.0, stack))
        .color(color)
        .border_radius(0.0)
        .padding(0.0);

    if Some(path) == selected {
        header = header
            .border_bottom(2.0)
            .border_color(style(Palette::PRIMARY));
    }

    let path = path.to_owned();
    let header = on_click(header, move |_, (_, state): &mut (_, DebugState)| {
        state.selected_tree = Some(path.clone());
    });

    debug_tree_hightlight(tree, state, header)
}

fn debug_tree_content(
    tree: &DebugTree,
    state: &mut TreeState,
    selected: Option<&[usize]>,
    path: &mut Vec<usize>,
) -> impl View<(DebugData, DebugState)> {
    (state.content).resize_with(tree.content().count(), TreeState::default);

    let mut content = vstack_any().align_items(Align::Start);

    for (i, child) in tree.content().enumerate() {
        let child_state = &mut state.content[i];
        child_state.transform = state.transform * child.transform();

        path.push(i);
        content.push(debug_tree_node(child, child_state, selected, path));
        path.pop();
    }

    hstack![expand(1.0, content)]
}

fn debug_tree_node(
    tree: &DebugTree,
    state: &mut TreeState,
    selected: Option<&[usize]>,
    path: &mut Vec<usize>,
) -> impl View<(DebugData, DebugState)> {
    let header = debug_tree_header(tree, state, selected, path);

    let content = if state.expanded {
        Some(debug_tree_content(tree, state, selected, path))
    } else {
        None
    };

    vstack![header, content].align_items(Align::Stretch)
}

fn debug_tree(data: &mut DebugData, state: &mut DebugState) -> impl View<(DebugData, DebugState)> {
    let root = data.tree.get_child(0).unwrap_or(&data.tree);

    let tree = debug_tree_node(
        root,
        &mut state.tree,
        state.selected_tree.as_deref(),
        &mut Vec::new(),
    );

    container(vscroll(tree))
}

fn selected_tree<'a>(data: &'a DebugData, state: &DebugState) -> Option<&'a DebugTree> {
    let root = data.tree.get_child(0)?;
    root.get_path(state.selected_tree.as_ref()?)
}

fn tree_time(event: &str, time: Option<Duration>) -> impl View<(DebugData, DebugState)> {
    match time {
        Some(time) => debug_text!("{} time: {:?}", event, time),
        None => debug_text!("No {} time", event),
    }
}

fn debug_selected_tree_group(
    name: &str,
    content: impl View<(DebugData, DebugState)>,
) -> impl View<(DebugData, DebugState)> {
    let header = debug_text!("{}", name);
    let content = pad([2.0, 0.0, 0.0, 12.0], content);

    collapsing(width(FILL, left(header)), content).icon_size(12.0)
}

fn debug_tree_performance(tree: &DebugTree) -> impl View<(DebugData, DebugState)> {
    vstack![
        tree_time("build", tree.build_time()),
        tree_time("rebuild", tree.rebuild_time()),
        tree_time("event", tree.event_time()),
        tree_time("layout", tree.layout_time()),
        tree_time("draw", tree.draw_time()),
    ]
    .align_items(Align::Start)
}

fn debug_tree_layout(tree: &DebugTree) -> impl View<(DebugData, DebugState)> {
    let transform = tree.transform();

    let offset = debug_text!("Offset: {}", transform.translation);
    let size = debug_text!("Size: {}", tree.rect().size());
    let min_size = debug_text!("Min size: {}", tree.space().min);
    let max_size = debug_text!("Max size: {}", tree.space().max);
    let flex = debug_text!("Flex: {}, {}", tree.flex_shrink(), tree.flex_grow());
    let depth = debug_text!("Depth: {}", tree.depth());

    vstack![offset, size, min_size, max_size, flex, depth].align_items(Align::Start)
}

fn debug_selected_tree(tree: &DebugTree) -> impl View<(DebugData, DebugState)> {
    let performance = debug_selected_tree_group("Performance", debug_tree_performance(tree));
    let layout = debug_selected_tree_group("Layout", debug_tree_layout(tree));

    top_left(vstack![performance, layout].gap(4.0))
}

fn debug_inspector_right_panel(
    data: &mut DebugData,
    state: &mut DebugState,
) -> impl View<(DebugData, DebugState)> {
    let content = selected_tree(data, state).map(debug_selected_tree);
    let content = width(350.0, pad(8.0, vscroll(content)));

    container(content)
}

fn debug_inspector(
    data: &mut DebugData,
    state: &mut DebugState,
) -> impl View<(DebugData, DebugState)> {
    hstack![
        expand(1.0, debug_tree(data, state)),
        debug_inspector_right_panel(data, state),
    ]
    .align_items(Align::Start)
    .gap(1.0)
}

fn average_time<T>(event: &str, time: Option<Duration>) -> impl View<T> {
    match time {
        Some(time) => debug_text!("Average {} time: {:?}", event, time),
        None => debug_text!("No {} time", event),
    }
}

fn debug_profiler(
    data: &mut DebugData,
    _state: &mut DebugState,
) -> impl View<(DebugData, DebugState)> {
    let stack = vstack![
        average_time("build", data.history.average_build_time()),
        average_time("rebuild", data.history.average_rebuild_time()),
        average_time("event", data.history.average_event_time()),
        average_time("layout", data.history.average_layout_time()),
        average_time("draw", data.history.average_draw_time()),
    ]
    .align_items(Align::Start);

    container(pad(8.0, hstack![stack]))
}

fn debug_bar_button<T>(content: impl View<T>) -> Button<impl View<T>> {
    button(content)
        .color(style(Palette::SECONDARY))
        .border_radius(0.0)
        .padding(0.0)
}

fn debug_tab(state: &mut DebugState, tab: DebugTab) -> impl View<(DebugData, DebugState)> {
    let content = debug_text!("{:?}", tab);

    let mut button = debug_bar_button(pad(4.0, content)).border_color(style(Palette::PRIMARY));

    if tab == state.tab {
        button = button.border_bottom(4.0);
    }

    on_click(
        height(24.0, button),
        move |_, (_, state): &mut (_, DebugState)| {
            state.tab = tab;
        },
    )
}

fn debug_close_button() -> impl View<(DebugData, DebugState)> {
    let content = text("✕").font_family(FontFamily::Monospace);
    let button = debug_bar_button(content);

    on_click(size(24.0, button), |_, (data, _): &mut (DebugData, _)| {
        data.is_open = false;
    })
}

fn debug_bar(state: &mut DebugState) -> impl View<(DebugData, DebugState)> {
    let tabs = hstack![
        debug_tab(state, DebugTab::Inspector),
        debug_tab(state, DebugTab::Profiler)
    ];

    let stack = hstack![tabs, debug_close_button()].justify_content(Justify::SpaceBetween);

    container(stack)
}

fn debug_panel(data: &mut DebugData, state: &mut DebugState) -> impl View<(DebugData, DebugState)> {
    match state.tab {
        DebugTab::Inspector => any(debug_inspector(data, state)),
        DebugTab::Profiler => any(debug_profiler(data, state)),
    }
}

fn debug(_data: &mut DebugData) -> impl View<DebugData> {
    with_state(DebugState::default, |data, state| {
        let stack = vstack![debug_bar(state), expand(1.0, debug_panel(data, state))].gap(1.0);

        let container = container(pad_top(1.0, stack)).background(style(Palette::SECONDARY_DARK));

        size([FILL, window_size().height / 3.0], container)
    })
}

/// Wrap a view in a debug view.
pub fn debug_ui<T>(content: impl View<T>) -> impl View<T> {
    DebugView::new(content, debug)
}
