use std::f32::consts::{FRAC_PI_2, PI, TAU};

use ori_macro::Build;

use crate::{
    canvas::{Canvas, Color, Curve, Mesh, Vertex},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Point, Rect, Size, Space, Vector},
    rebuild::Rebuild,
    style::{style, Style, Styles},
    view::View,
};

/// Create a new [`ColorPicker`].
pub fn color_picker<T>() -> ColorPicker<T> {
    ColorPicker::new()
}

/// The style of a color picker.
#[derive(Clone, Debug)]
pub struct ColorPickerStyle {
    /// The size of the color picker.
    pub size: f32,

    /// The border width of the color picker.
    pub border_width: f32,

    /// The border color of the color picker.
    pub border_color: Color,

    /// The width of the sliders.
    pub slider_width: f32,

    /// The padding of the sliders.
    pub slider_padding: f32,

    /// The color of the slider tracks.
    pub track_color: Color,

    /// The color of the lightness slider.
    pub lightness_color: Color,

    /// The color of the alpha slider.
    pub alpha_color: Color,
}

impl Style for ColorPickerStyle {
    fn style(style: &Styles) -> Self {
        let palette = style.palette();

        Self {
            size: 140.0,
            border_width: 2.0,
            border_color: palette.surface_higher,
            slider_width: 8.0,
            slider_padding: 8.0,
            track_color: palette.surface_higher,
            lightness_color: palette.primary,
            alpha_color: palette.secondary,
        }
    }
}

/// A color picker.
#[derive(Build, Rebuild)]
pub struct ColorPicker<T> {
    /// The color of the color picker.
    #[rebuild(draw)]
    pub color: Color,

    /// The on_input callback.
    #[build(ignore)]
    #[allow(clippy::type_complexity)]
    pub on_input: Option<Box<dyn FnMut(&mut EventCx, &mut T, Color)>>,

    /// The size of the color picker.
    #[rebuild(layout)]
    pub size: f32,

    /// The border width of the color picker.
    #[rebuild(draw)]
    pub border_width: f32,

    /// The border color of the color picker.
    #[rebuild(draw)]
    pub border_color: Color,

    /// The width of the sliders.
    #[rebuild(draw)]
    pub slider_width: f32,

    /// The padding of the sliders.
    #[rebuild(draw)]
    pub slider_padding: f32,

    /// The color of the slider tracks.
    #[rebuild(draw)]
    pub track_color: Color,

    /// The color of the lightness slider.
    #[rebuild(draw)]
    pub lightness_color: Color,

    /// The color of the alpha slider.
    #[rebuild(draw)]
    pub alpha_color: Color,
}

impl<T> Default for ColorPicker<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ColorPicker<T> {
    const SLIDER_HALF: f32 = FRAC_PI_2 * 0.95;
    const SLIDER_ARC: f32 = Self::SLIDER_HALF * 2.0;
    const SLIDER_SHIM: f32 = FRAC_PI_2 - Self::SLIDER_HALF;

    /// Create a new [`ColorPicker`].
    pub fn new() -> Self {
        Self::styled(style())
    }

    /// Create a new [`ColorPicker`] with the given style.
    pub fn styled(style: ColorPickerStyle) -> Self {
        Self {
            color: Color::okhsl(0.0, 0.0, 0.5),
            on_input: None,
            size: style.size,
            border_width: style.border_width,
            border_color: style.border_color,
            slider_width: style.slider_width,
            slider_padding: style.slider_padding,
            track_color: style.track_color,
            lightness_color: style.lightness_color,
            alpha_color: style.alpha_color,
        }
    }

    /// Set the on_input callback.
    pub fn on_input(mut self, on_input: impl FnMut(&mut EventCx, &mut T, Color) + 'static) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    fn wheel_mesh(lightness: f32, alpha: f32, radial: u32, angular: u32) -> Mesh {
        let mut mesh = Mesh::new();

        mesh.vertices.push(Vertex {
            position: Point::ZERO,
            tex_coords: Point::ZERO,
            color: Color::okhsla(0.0, 0.0, lightness, alpha),
        });

        for i in 1..=radial {
            let r = i as f32 / radial as f32;

            for j in 0..angular {
                let a = j as f32 / angular as f32 * TAU;

                mesh.vertices.push(Vertex {
                    position: Point::new(r * a.cos(), r * a.sin()),
                    tex_coords: Point::ZERO,
                    color: Color::okhsla(a.to_degrees(), r, lightness, alpha),
                });

                let di0 = j;
                let di1 = (j + 1) % angular;

                let i2 = (i - 1) * angular + di0;
                let i3 = (i - 1) * angular + di1;

                if i == 1 {
                    mesh.indices.push(0);
                    mesh.indices.push(i3);
                    mesh.indices.push(i2);
                } else {
                    let i0 = (i - 2) * angular + di0;
                    let i1 = (i - 2) * angular + di1;

                    mesh.indices.push(i0);
                    mesh.indices.push(i1);
                    mesh.indices.push(i2);

                    mesh.indices.push(i1);
                    mesh.indices.push(i3);
                    mesh.indices.push(i2);
                }
            }
        }

        mesh
    }

    fn input(
        &mut self,
        state: &mut ColorPickerState,
        cx: &mut EventCx,
        data: &mut T,
        position: Point,
    ) {
        let local = cx.local(position) - cx.rect().center();
        let angle = local.angle();
        let radius = cx.size().min_element() / 2.0;
        let wheel_radius = radius - self.slider_width - self.slider_padding;
        let slider_start = radius - self.slider_width;
        let slider_radius = radius - self.slider_width / 2.0;

        let (h, s, l, a) = self.color.to_okhsla();

        if state.can_edit(ColorPickerPart::Wheel, local.length() <= wheel_radius) {
            state.edit = Some(ColorPickerPart::Wheel);

            let hue = angle.to_degrees();
            let saturation = f32::clamp(local.length() / wheel_radius, 0.0, 1.0);

            let color = Color::okhsla(hue, saturation, l, a);

            if let Some(ref mut on_input) = self.on_input {
                on_input(cx, data, color);
                cx.request_rebuild();
            }

            return;
        }

        let is_slider = (slider_start..=radius).contains(&local.length());
        let slider_height = f32::clamp(local.y / slider_radius, -1.0, 1.0);
        let slider_angle = if slider_height == -1.0 {
            PI
        } else if slider_height == 1.0 {
            0.0
        } else {
            slider_height.acos()
        };

        if state.can_edit(
            ColorPickerPart::Alpha,
            (-FRAC_PI_2..=FRAC_PI_2).contains(&angle) && is_slider,
        ) {
            state.edit = Some(ColorPickerPart::Alpha);

            let alpha = (slider_angle - Self::SLIDER_SHIM) / Self::SLIDER_ARC;
            let alpha = alpha.clamp(0.0, 1.0);

            let color = Color::okhsla(h, s, l, alpha);

            if let Some(ref mut on_input) = self.on_input {
                on_input(cx, data, color);
                cx.request_rebuild();
            }
        } else if state.can_edit(ColorPickerPart::Lightness, is_slider) {
            state.edit = Some(ColorPickerPart::Lightness);

            let lightness = (slider_angle - Self::SLIDER_SHIM) / Self::SLIDER_ARC;
            let lightness = lightness.clamp(0.001, 0.999);

            let color = Color::okhsla(h, s, lightness, a);

            if let Some(ref mut on_input) = self.on_input {
                on_input(cx, data, color);
                cx.request_rebuild();
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorPickerPart {
    Wheel,
    Lightness,
    Alpha,
}

#[doc(hidden)]
pub struct ColorPickerState {
    mesh: Mesh,
    edit: Option<ColorPickerPart>,
}

impl ColorPickerState {
    fn can_edit(&self, part: ColorPickerPart, inside: bool) -> bool {
        self.edit.map_or(inside, |edit| edit == part)
    }
}

impl<T> View<T> for ColorPicker<T> {
    type State = ColorPickerState;

    fn build(&mut self, _cx: &mut BuildCx, _data: &mut T) -> Self::State {
        let (_, _, l, a) = self.color.to_okhsla();

        ColorPickerState {
            mesh: Self::wheel_mesh(l, a, 180, 50),
            edit: None,
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);

        let (_, _, l, a) = self.color.to_okhsla();
        let (_, _, old_l, old_a) = old.color.to_okhsla();

        if (l - old_l).abs() < 0.01 || (a - old_a).abs() < 0.01 {
            state.mesh = Self::wheel_mesh(l, a, 180, 50);
        }
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        match event {
            Event::PointerPressed(e) if cx.is_hot() => {
                self.input(state, cx, data, e.position);
                cx.set_active(true);
            }
            Event::PointerMoved(e) if cx.is_active() => {
                self.input(state, cx, data, e.position);
            }
            Event::PointerReleased(_) => {
                cx.set_active(false);
                state.edit = None;
            }
            _ => {}
        }
    }

    fn layout(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        space.fit(Size::all(self.size))
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        _data: &mut T,
        canvas: &mut Canvas,
    ) {
        canvas.set_hoverable(cx.id());

        let radius = cx.size().min_element() / 2.0;
        let wheel_radius = radius - self.slider_width - self.slider_padding;

        canvas.translate(cx.rect().center().to_vector());

        canvas.draw_quad(
            Rect::center_size(
                Point::ZERO,
                Size::all(wheel_radius * 2.0 + self.border_width * 2.0),
            ),
            Color::TRANSPARENT,
            wheel_radius + self.border_width,
            self.border_width,
            self.border_color,
        );

        {
            let mut canvas = canvas.layer();
            canvas.scale(Vector::all(wheel_radius));
            canvas.draw(state.mesh.clone());
        }

        let mut canvas = canvas.layer();
        let (h, s, l, a) = self.color.to_okhsla();

        let offset = Vector::from_angle(h.to_radians()) * s * wheel_radius;

        canvas.draw_quad(
            Rect::center_size(Point::ZERO + offset, Size::all(8.0)),
            Color::TRANSPARENT,
            4.0,
            1.0,
            Color::WHITE,
        );

        canvas.draw_quad(
            Rect::center_size(Point::ZERO + offset, Size::all(12.0)),
            Color::TRANSPARENT,
            6.0,
            2.0,
            Color::BLACK,
        );

        // draw the lightness slider from bottom to top along the left edge

        let slider_radius = radius - self.slider_width / 2.0;

        let track = Curve::arc_center_angle(
            Point::ZERO,
            slider_radius,
            PI - Self::SLIDER_HALF,
            PI + Self::SLIDER_HALF,
        );

        canvas.draw(track.stroke(self.slider_width, self.track_color));

        let slider = Curve::arc_center_angle(
            Point::ZERO,
            slider_radius,
            PI - Self::SLIDER_HALF,
            PI - Self::SLIDER_HALF + Self::SLIDER_ARC * l,
        );

        canvas.draw(slider.stroke(self.slider_width, self.lightness_color));

        // draw the alpha slider from left to right along the bottom edge

        let track = Curve::arc_center_angle(
            Point::ZERO,
            slider_radius,
            -Self::SLIDER_HALF,
            Self::SLIDER_HALF,
        );

        canvas.draw(track.stroke(self.slider_width, self.track_color));

        let slider = Curve::arc_center_angle(
            Point::ZERO,
            slider_radius,
            Self::SLIDER_HALF - Self::SLIDER_ARC * a,
            Self::SLIDER_HALF,
        );

        canvas.draw(slider.stroke(self.slider_width, self.alpha_color));
    }
}
