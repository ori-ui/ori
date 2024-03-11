use std::time::Duration;

use crate::{
    canvas::{Background, BorderRadius, BorderWidth, Color, Fragment, Primitive, Quad},
    layout::{Affine, Align, Justify, Point, Rect, FILL},
    text::FontFamily,
    theme::{style, window_size, Palette},
    view::{any, View},
    views::{
        button, container, flex, height, hstack, on_click, on_draw, pad, pad_left, pad_top, size,
        text, trigger, vscroll, vstack, vstack_any, width, with_state, Button,
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
}

fn debug_tree_header(
    tree: &DebugTree,
    state: &mut TreeState,
    path: &[usize],
) -> impl View<(DebugData, DebugState)> {
    let expand = if tree.content().count() > 0 {
        let symbol = if state.expanded { "-" } else { "+" };

        let symbol = text(symbol)
            .font_size(12.0)
            .font_family(FontFamily::Monospace);

        let path = path.to_owned();
        let expand = on_click(
            trigger(symbol),
            move |_, (_, state): &mut (_, DebugState)| {
                let tree_state = state.tree.get_mut(&path);

                tree_state.expanded = !tree_state.expanded;
            },
        );

        Ok(width(12.0, expand))
    } else {
        Err(width(12.0, ()))
    };

    let name = text(tree.short_name())
        .font_size(12.0)
        .font_family(FontFamily::Monospace);

    let rect = tree.rect();
    let transform = state.transform;

    let size = text(format!("[{} x {}]", rect.width(), rect.height()))
        .font_size(12.0)
        .font_family(FontFamily::Monospace);

    let stack = hstack![expand, name, size]
        .align_items(Align::Start)
        .gap(4.0);

    let header = button(pad_left(path.len() as f32 * 10.0 + 20.0, stack))
        .color(style(Palette::SECONDARY))
        .border_radius(0.0)
        .padding(0.0);

    on_draw(trigger(width(FILL, header)), move |cx, _, canvas| {
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

fn debug_tree_content(
    tree: &DebugTree,
    state: &mut TreeState,
    path: &mut Vec<usize>,
) -> impl View<(DebugData, DebugState)> {
    (state.content).resize_with(tree.content().count(), TreeState::default);

    let mut content = vstack_any().align_items(Align::Start);

    for (i, child) in tree.content().enumerate() {
        let child_state = &mut state.content[i];
        child_state.transform = state.transform * child.transform();

        path.push(i);
        content.push(debug_tree_node(child, child_state, path));
        path.pop();
    }

    hstack![flex(1.0, content)]
}

fn debug_tree_node(
    tree: &DebugTree,
    state: &mut TreeState,
    path: &mut Vec<usize>,
) -> impl View<(DebugData, DebugState)> {
    let header = debug_tree_header(tree, state, path);

    let content = if state.expanded {
        Some(debug_tree_content(tree, state, path))
    } else {
        None
    };

    vstack![header, content].align_items(Align::Start)
}

fn debug_tree(data: &mut DebugData, state: &mut DebugState) -> impl View<(DebugData, DebugState)> {
    let root = data.tree.get_child(0).unwrap_or(&data.tree);
    let tree = debug_tree_node(root, &mut state.tree, &mut Vec::new());
    vscroll(tree)
}

fn debug_inspector(
    data: &mut DebugData,
    state: &mut DebugState,
) -> impl View<(DebugData, DebugState)> {
    debug_tree(data, state)
}

fn average_time<T>(event: &str, time: Option<Duration>) -> impl View<T> {
    let time = match time {
        Some(time) => format!("Average {} time: {:?}", event, time),
        None => format!("No {} time", event),
    };

    text(time)
        .font_size(12.0)
        .font_family(FontFamily::Monospace)
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

    pad(8.0, stack)
}

fn debug_bar_button<T>(content: impl View<T>) -> Button<impl View<T>> {
    button(content)
        .color(style(Palette::SECONDARY))
        .border_radius(0.0)
        .padding(0.0)
}

fn debug_tab(state: &mut DebugState, tab: DebugTab) -> impl View<(DebugData, DebugState)> {
    let content = text(format!("{:?}", tab))
        .font_family(FontFamily::Monospace)
        .font_size(12.0);

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

    let items = hstack![tabs, debug_close_button()].justify_content(Justify::SpaceBetween);
    width(FILL, items)
}

fn debug_panel(data: &mut DebugData, state: &mut DebugState) -> impl View<(DebugData, DebugState)> {
    let content = match state.tab {
        DebugTab::Inspector => any(debug_inspector(data, state)),
        DebugTab::Profiler => any(debug_profiler(data, state)),
    };

    container(pad_top(1.0, content))
        .background(style(Palette::SECONDARY))
        .border_color(style(Palette::SECONDARY_DARK))
        .border_top(1.0)
}

fn debug(_data: &mut DebugData) -> impl View<DebugData> {
    with_state(DebugState::default, |data, state| {
        let stack = vstack![
            debug_bar(state),
            flex(1.0, size(FILL, debug_panel(data, state)))
        ];

        let container = container(pad_top(1.0, stack))
            .background(style(Palette::SECONDARY))
            .border_color(style(Palette::SECONDARY_DARK))
            .border_top(1.0);

        size([FILL, window_size().height / 3.0], container)
    })
}

/// Wrap a view in a debug view.
pub fn debug_ui<T>(content: impl View<T>) -> impl View<T> {
    DebugView::new(content, debug)
}
