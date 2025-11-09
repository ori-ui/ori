use gtk4::prelude::{GtkWindowExt as _, WidgetExt as _};

use crate::{Context, View};

pub fn window<V>(content: V) -> Window<V> {
    Window::new(content)
}

#[allow(unused)]
#[derive(Debug)]
pub struct Window<V> {
    pub(crate) content: V,
    pub(crate) id: Option<ori::ViewId>,
    pub(crate) title: String,
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
    pub(crate) visible: bool,
    pub(crate) resizable: bool,
    pub(crate) decorated: bool,
    pub(crate) hide_on_close: bool,

    #[cfg(feature = "layer-shell")]
    pub(crate) is_layer_shell: bool,
    #[cfg(feature = "layer-shell")]
    pub(crate) layer: Layer,
    #[cfg(feature = "layer-shell")]
    pub(crate) exclusive_zone: Option<i32>,
    #[cfg(feature = "layer-shell")]
    pub(crate) margin_top: i32,
    #[cfg(feature = "layer-shell")]
    pub(crate) margin_right: i32,
    #[cfg(feature = "layer-shell")]
    pub(crate) margin_bottom: i32,
    #[cfg(feature = "layer-shell")]
    pub(crate) margin_left: i32,
    #[cfg(feature = "layer-shell")]
    pub(crate) anchor_top: bool,
    #[cfg(feature = "layer-shell")]
    pub(crate) anchor_right: bool,
    #[cfg(feature = "layer-shell")]
    pub(crate) anchor_bottom: bool,
    #[cfg(feature = "layer-shell")]
    pub(crate) anchor_left: bool,
}

#[cfg(feature = "layer-shell")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

impl<V> Window<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            id: None,
            title: String::from("Ori Gtk4 App"),
            width: None,
            height: None,
            visible: true,
            resizable: true,
            decorated: true,
            hide_on_close: false,

            #[cfg(feature = "layer-shell")]
            is_layer_shell: false,
            #[cfg(feature = "layer-shell")]
            layer: Layer::Top,
            #[cfg(feature = "layer-shell")]
            exclusive_zone: None,
            #[cfg(feature = "layer-shell")]
            margin_top: 0,
            #[cfg(feature = "layer-shell")]
            margin_right: 0,
            #[cfg(feature = "layer-shell")]
            margin_bottom: 0,
            #[cfg(feature = "layer-shell")]
            margin_left: 0,
            #[cfg(feature = "layer-shell")]
            anchor_top: false,
            #[cfg(feature = "layer-shell")]
            anchor_right: false,
            #[cfg(feature = "layer-shell")]
            anchor_bottom: false,
            #[cfg(feature = "layer-shell")]
            anchor_left: false,
        }
    }

    pub fn title(mut self, title: impl ToString) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn width(mut self, width: impl Into<Option<u32>>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Option<u32>>) -> Self {
        self.height = height.into();
        self
    }

    pub fn size(
        self,
        width: impl Into<Option<u32>>,
        height: impl Into<Option<u32>>,
    ) -> Self {
        self.width(width).height(height)
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn decorated(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }

    pub fn hide_on_close(mut self, hide_on_close: bool) -> Self {
        self.hide_on_close = hide_on_close;
        self
    }

    /// Makes window a layer shell.
    #[cfg(feature = "layer-shell")]
    pub fn is_layer_shell(mut self, is_layer_shell: bool) -> Self {
        self.is_layer_shell = is_layer_shell;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn exclusive_zone(mut self, zone: impl Into<Option<i32>>) -> Self {
        self.exclusive_zone = zone.into();
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn margin_top(mut self, margin: i32) -> Self {
        self.margin_top = margin;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn margin_right(mut self, margin: i32) -> Self {
        self.margin_right = margin;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn margin_bottom(mut self, margin: i32) -> Self {
        self.margin_bottom = margin;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn margin_left(mut self, margin: i32) -> Self {
        self.margin_left = margin;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn anchor_top(mut self, anchor: bool) -> Self {
        self.anchor_top = anchor;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn anchor_right(mut self, anchor: bool) -> Self {
        self.anchor_right = anchor;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn anchor_bottom(mut self, anchor: bool) -> Self {
        self.anchor_bottom = anchor;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn anchor_left(mut self, anchor: bool) -> Self {
        self.anchor_left = anchor;
        self
    }
}

#[cfg(feature = "layer-shell")]
impl From<Layer> for gtk4_layer_shell::Layer {
    fn from(layer: Layer) -> Self {
        match layer {
            Layer::Background => gtk4_layer_shell::Layer::Background,
            Layer::Bottom => gtk4_layer_shell::Layer::Bottom,
            Layer::Top => gtk4_layer_shell::Layer::Top,
            Layer::Overlay => gtk4_layer_shell::Layer::Overlay,
        }
    }
}

pub struct WindowState<T, V>
where
    V: View<T>,
{
    window: gtk4::ApplicationWindow,
    child: V::Element,
    state: V::State,
}

impl<T, V> ori::View<Context, T> for Window<V>
where
    V: View<T>,
{
    type Element = ori::NoElement;
    type State = WindowState<T, V>;

    fn build(
        &mut self,
        cx: &mut Context,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
        let (child, state) = self.content.build(cx, data);

        let window = gtk4::ApplicationWindow::default();

        if let Some(app) = cx.app().upgrade() {
            window.set_application(Some(&app));
        }

        window.set_child(Some(&child));
        set_state(&window, self);

        window.present();

        let state = WindowState {
            window,
            child,
            state,
        };

        (ori::NoElement, state)
    }

    fn rebuild(
        &mut self,
        _element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) {
        self.content.rebuild(
            &mut state.child,
            &mut state.state,
            cx,
            data,
            &mut old.content,
        );

        if !super::is_parent(&state.window, &state.child) {
            state.window.set_child(Some(&state.child));
        }

        update_state(&state.window, self, old);
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        state: Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.teardown(state.child, state.state, cx, data);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        self.content.event(
            &mut state.child,
            &mut state.state,
            cx,
            data,
            event,
        )
    }
}

fn set_state<V>(win: &gtk4::ApplicationWindow, desc: &Window<V>) {
    #[cfg(feature = "layer-shell")]
    if desc.is_layer_shell {
        use gtk4_layer_shell::{Edge, LayerShell as _};

        win.init_layer_shell();
        win.set_layer(desc.layer.into());

        if let Some(zone) = desc.exclusive_zone {
            win.set_exclusive_zone(zone);
        } else {
            win.auto_exclusive_zone_enable();
        }

        win.set_anchor(Edge::Top, desc.anchor_top);
        win.set_anchor(Edge::Right, desc.anchor_right);
        win.set_anchor(Edge::Bottom, desc.anchor_bottom);
        win.set_anchor(Edge::Left, desc.anchor_left);

        win.set_margin(Edge::Top, desc.margin_top);
        win.set_margin(Edge::Right, desc.margin_right);
        win.set_margin(Edge::Bottom, desc.margin_bottom);
        win.set_margin(Edge::Left, desc.margin_left);
    }

    win.set_title(Some(&desc.title));
    win.set_default_size(
        desc.width.map_or(-1, |width| width as i32),
        desc.height.map_or(-1, |width| width as i32),
    );

    win.set_visible(desc.visible);
    win.set_resizable(desc.resizable);
    win.set_decorated(desc.decorated);
    win.set_hide_on_close(desc.hide_on_close);
}

fn update_state<V>(
    win: &gtk4::ApplicationWindow,
    desc: &Window<V>,
    old: &Window<V>,
) {
    #[cfg(feature = "layer-shell")]
    if desc.is_layer_shell {
        use gtk4_layer_shell::{Edge, LayerShell as _};

        if desc.layer != old.layer {
            win.set_layer(desc.layer.into());
        }

        if desc.exclusive_zone != old.exclusive_zone {
            if let Some(zone) = desc.exclusive_zone {
                win.set_exclusive_zone(zone);
            } else {
                win.auto_exclusive_zone_enable();
            }
        }

        if desc.anchor_top != old.anchor_top {
            win.set_anchor(Edge::Top, desc.anchor_top);
        }

        if desc.anchor_right != old.anchor_right {
            win.set_anchor(Edge::Right, desc.anchor_right);
        }

        if desc.anchor_bottom != old.anchor_bottom {
            win.set_anchor(Edge::Bottom, desc.anchor_bottom);
        }

        if desc.anchor_left != old.anchor_left {
            win.set_anchor(Edge::Left, desc.anchor_left);
        }

        if desc.margin_top != old.margin_top {
            win.set_margin(Edge::Top, desc.margin_top);
        }

        if desc.margin_right != old.margin_right {
            win.set_margin(Edge::Right, desc.margin_right);
        }

        if desc.margin_bottom != old.margin_bottom {
            win.set_margin(Edge::Bottom, desc.margin_bottom);
        }

        if desc.margin_left != old.margin_left {
            win.set_margin(Edge::Left, desc.margin_left);
        }
    }

    if desc.title != old.title {
        win.set_title(Some(&desc.title));
    }

    if desc.width != old.width || desc.height != old.height {
        win.set_default_size(
            desc.width.map_or(-1, |width| width as i32),
            desc.height.map_or(-1, |width| width as i32),
        );
    }

    if desc.visible != old.visible {
        win.set_visible(desc.visible);
    }

    if desc.resizable != old.resizable {
        win.set_resizable(desc.resizable);
    }

    if desc.decorated != old.decorated {
        win.set_decorated(desc.decorated);
    }

    if desc.hide_on_close != old.hide_on_close {
        win.set_hide_on_close(desc.hide_on_close);
    }
}
