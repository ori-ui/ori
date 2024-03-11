use crate::{
    canvas::{Background, BorderRadius, BorderWidth, Color, Fragment, Primitive, Quad},
    layout::{Affine, Align, Justify, Point, Rect, FILL},
    text::FontFamily,
    theme::{style, window_size, Palette},
    view::View,
    views::{
        button, container, flex, focus, hstack, on_click, on_draw, pad, pad_left, pad_top, size,
        text, trigger, vscroll, vstack, vstack_any, width, with_state, without_state,
    },
};

use super::{DebugData, DebugTree, DebugView};

#[derive(Debug, Default)]
struct DebugTreeState {
    expanded: bool,
}

fn debug_tree_header(
    tree: &DebugTree,
    state: &mut DebugTreeState,
    depth: u32,
    transform: Affine,
) -> impl View<(DebugTree, DebugTreeState)> {
    let expand = if tree.content().count() > 0 {
        let symbol = if state.expanded { "-" } else { "+" };

        let symbol = text(symbol)
            .font_size(12.0)
            .font_family(FontFamily::Monospace);

        let expand = on_click(
            trigger(symbol),
            |_, (_, state): &mut (_, DebugTreeState)| {
                state.expanded = !state.expanded;
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
    let transform = transform * tree.transform();

    let size = text(format!("[{} x {}]", rect.width(), rect.height()))
        .font_size(12.0)
        .font_family(FontFamily::Monospace);

    let stack = hstack![expand, name, size]
        .align_items(Align::Start)
        .gap(4.0);

    let header = button(pad_left(depth as f32 * 10.0 + 20.0, stack))
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
    _state: &mut DebugTreeState,
    depth: u32,
    transform: Affine,
) -> impl View<(DebugTree, DebugTreeState)> {
    let mut content = vstack_any().align_items(Align::Start);

    for i in 0..tree.content().count() {
        let transform = transform * tree.transform();

        content.push(without_state(debug_tree_node(i, depth + 1, transform)));
    }

    hstack![flex(1.0, content)]
}

fn debug_tree_node(child: usize, depth: u32, transform: Affine) -> impl View<DebugTree> {
    focus(
        move |tree: &mut DebugTree, lens| {
            if let Some(child) = tree.get_child_mut(child) {
                lens(child)
            } else {
                lens(tree)
            }
        },
        with_state(DebugTreeState::default, move |tree, state| {
            let header = debug_tree_header(tree, state, depth, transform);

            let content = if state.expanded {
                Some(debug_tree_content(tree, state, depth, transform))
            } else {
                None
            };

            vstack![header, content].align_items(Align::Start)
        }),
    )
}

fn debug_tree() -> impl View<DebugData> {
    let tree = focus(
        |data: &mut DebugData, lens| lens(&mut data.tree),
        debug_tree_node(0, 0, Affine::IDENTITY),
    );

    container(pad(1.0, vscroll(tree)))
        .border_width([1.0, 1.0, 0.0, 0.0])
        .border_color(style(Palette::SECONDARY_DARK))
}

fn debugger_bar_button<T>(content: impl View<T>) -> impl View<T> {
    button(content)
        .color(style(Palette::SECONDARY))
        .border_radius(0.0)
        .padding(0.0)
}

fn debugger_bar() -> impl View<DebugData> {
    let close_button = on_click(
        debugger_bar_button(size(24.0, text("✕"))),
        |_, data: &mut DebugData| {
            data.is_open = false;
        },
    );

    let items = hstack![(), close_button].justify_content(Justify::SpaceBetween);
    width(FILL, items)
}

fn debugger(_data: &mut DebugData) -> impl View<DebugData> {
    let stack = vstack![debugger_bar(), flex(1.0, debug_tree())];

    let container = container(pad_top(1.0, stack))
        .border_top(1.0)
        .background(style(Palette::SECONDARY))
        .border_color(style(Palette::SECONDARY_DARK));

    size([FILL, window_size().height / 3.0], container)
}

/// Wrap a view in a debug view.
pub fn debug_ui<T>(content: impl View<T>) -> impl View<T> {
    DebugView::new(content, debugger)
}
