use std::f32::consts::{FRAC_PI_2, PI, TAU};

use ori_macro::Build;

use crate::{
    canvas::Color,
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Affine, Point, Rect, Size, Space, Vector},
    prelude::{Curve, FillRule, Image, Pattern},
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
            slider_width: 12.0,
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
    const SLIDER_HALF: f32 = FRAC_PI_2 * 0.9;
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
            lightness_color: style.lightness_color,
            alpha_color: style.alpha_color,
        }
    }

    /// Set the on_input callback.
    pub fn on_input(mut self, on_input: impl FnMut(&mut EventCx, &mut T, Color) + 'static) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    fn wheel_image(lightness: f32, alpha: f32) -> Image {
        let mut pixels = vec![0u8; 4 * 128 * 128];

        for y in 0..128 {
            for x in 0..128 {
                let angle = f32::atan2(y as f32 - 64.0, x as f32 - 64.0);
                let radius = f32::hypot(y as f32 - 64.0, x as f32 - 64.0);

                let hue = angle.to_degrees();
                let saturation = radius / 64.0;

                let color = Color::okhsla(hue, saturation, lightness, alpha);

                let i = (y * 128 + x) * 4;
                let [r, g, b, a] = color.to_rgba8();

                pixels[i] = (r as u16 * a as u16 / 255) as u8;
                pixels[i + 1] = (g as u16 * a as u16 / 255) as u8;
                pixels[i + 2] = (b as u16 * a as u16 / 255) as u8;
                pixels[i + 3] = a;
            }
        }

        Image::new(pixels, 128, 128)
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
        let wheel_radius = radius - self.slider_width;

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

        let slider_angle = (angle + TAU + FRAC_PI_2) % TAU;

        if state.can_edit(ColorPickerPart::Alpha, local.x > 2.0) && local.x > 2.0 {
            state.edit = Some(ColorPickerPart::Alpha);

            let alpha = (PI - Self::SLIDER_SHIM - slider_angle) / Self::SLIDER_ARC;
            let alpha = alpha.clamp(0.0, 1.0);

            let color = Color::okhsla(h, s, l, alpha);

            if let Some(ref mut on_input) = self.on_input {
                on_input(cx, data, color);
                cx.request_rebuild();
            }
        } else if state.can_edit(ColorPickerPart::Lightness, local.x < -2.0) && local.x < -2.0 {
            state.edit = Some(ColorPickerPart::Lightness);

            let lightness = (slider_angle - PI - Self::SLIDER_SHIM) / Self::SLIDER_ARC;
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
    image: Option<Image>,
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
        ColorPickerState {
            image: None,
            edit: None,
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);

        let (_, _, l, a) = self.color.to_okhsla();
        let (_, _, old_l, old_a) = old.color.to_okhsla();

        if (l - old_l).abs() > 1e-6 || (a - old_a).abs() > 1e-6 {
            state.image = None;
            cx.request_draw();
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

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, _data: &mut T) {
        let radius = cx.size().min_element() / 2.0;
        let wheel_radius = radius - self.slider_width;

        let (h, s, l, a) = self.color.to_okhsla();

        let image = state.image.get_or_insert_with(|| Self::wheel_image(l, a));

        cx.translate(cx.rect().center() - cx.rect().top_left(), |cx| {
            cx.hoverable(|cx| {
                // draw the sliders
                let lightness_angle = -Self::SLIDER_HALF + l * Self::SLIDER_ARC;
                let alpha_angle = Self::SLIDER_HALF - a * Self::SLIDER_ARC;

                cx.rotate(lightness_angle, |cx| {
                    cx.quad(
                        Rect::center_size(
                            Point::new(-wheel_radius, 0.0),
                            Size::new(self.slider_width * 2.0, self.slider_width * 1.5),
                        ),
                        self.lightness_color,
                        self.slider_width / 2.0,
                        self.border_width / 2.0,
                        self.border_color,
                    );
                });

                cx.rotate(alpha_angle, |cx| {
                    cx.quad(
                        Rect::center_size(
                            Point::new(wheel_radius, 0.0),
                            Size::new(self.slider_width * 2.0, self.slider_width * 1.5),
                        ),
                        self.alpha_color,
                        self.slider_width / 2.0,
                        self.border_width / 2.0,
                        self.border_color,
                    );
                });

                // draw the wheel
                cx.fill(
                    Curve::circle(Point::ZERO, wheel_radius + self.border_width),
                    FillRule::Winding,
                    self.border_color,
                );

                let pattern = Pattern {
                    image: image.clone(),
                    transform: Affine::translate(cx.rect().top_left() - cx.rect().center())
                        * Affine::scale(Vector::from(cx.size() / image.size())),
                    opacity: 1.0,
                };

                cx.fill(
                    Curve::circle(Point::ZERO, wheel_radius),
                    FillRule::Winding,
                    pattern,
                );

                let offset = Vector::from_angle(h.to_radians()) * s * wheel_radius;

                cx.quad(
                    Rect::center_size(Point::ZERO + offset, Size::all(8.0)),
                    Color::TRANSPARENT,
                    4.0,
                    1.0,
                    Color::WHITE,
                );

                cx.quad(
                    Rect::center_size(Point::ZERO + offset, Size::all(10.0)),
                    Color::TRANSPARENT,
                    5.0,
                    1.0,
                    Color::BLACK,
                );
            });
        });
    }
}
