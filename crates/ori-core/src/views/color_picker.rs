use std::f32::consts::{FRAC_PI_2, PI, TAU};

use ori_macro::{Build, Styled};

use crate::{
    canvas::{Color, Curve, FillRule, Pattern},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    image::Image,
    layout::{Affine, Point, Rect, Size, Space, Vector},
    rebuild::Rebuild,
    style::{Styled, ACCENT, OUTLINE, PRIMARY},
    view::View,
};

/// Create a new [`ColorPicker`].
pub fn color_picker<T>() -> ColorPicker<T> {
    ColorPicker::new()
}

/// A color picker.
#[derive(Styled, Build, Rebuild)]
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
    #[styled(default = 128.0)]
    pub size: Styled<f32>,

    /// The border width of the color picker.
    #[rebuild(draw)]
    #[styled(default = 2.0)]
    pub border_width: Styled<f32>,

    /// The border color of the color picker.
    #[rebuild(draw)]
    #[styled(default -> OUTLINE or Color::BLACK)]
    pub border_color: Styled<Color>,

    /// The width of the sliders.
    #[rebuild(draw)]
    #[styled(default = 12.0)]
    pub slider_width: Styled<f32>,

    /// The color of the lightness slider.
    #[rebuild(draw)]
    #[styled(default -> PRIMARY or Color::BLUE)]
    pub lightness_color: Styled<Color>,

    /// The color of the alpha slider.
    #[rebuild(draw)]
    #[styled(default -> ACCENT or Color::RED)]
    pub alpha_color: Styled<Color>,
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
        Self {
            color: Color::WHITE,
            on_input: None,
            size: ColorPickerStyle::SIZE.into(),
            border_width: ColorPickerStyle::BORDER_WIDTH.into(),
            border_color: ColorPickerStyle::BORDER_COLOR.into(),
            slider_width: ColorPickerStyle::SLIDER_WIDTH.into(),
            lightness_color: ColorPickerStyle::LIGHTNESS_COLOR.into(),
            alpha_color: ColorPickerStyle::ALPHA_COLOR.into(),
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
        let wheel_radius = radius - state.style.slider_width;

        let (h, s, l, a) = self.color.to_okhsla();

        if state.can_edit(ColorPickerPart::Wheel, local.length() <= wheel_radius) {
            state.edit = Some(ColorPickerPart::Wheel);

            let hue = angle.to_degrees();
            let saturation = f32::clamp(local.length() / wheel_radius, 0.0, 1.0);

            let color = Color::okhsla(hue, saturation, l, a);

            if let Some(ref mut on_input) = self.on_input {
                on_input(cx, data, color);
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
            }
        } else if state.can_edit(ColorPickerPart::Lightness, local.x < -2.0) && local.x < -2.0 {
            state.edit = Some(ColorPickerPart::Lightness);

            let lightness = (slider_angle - PI - Self::SLIDER_SHIM) / Self::SLIDER_ARC;
            let lightness = lightness.clamp(0.001, 0.999);

            let color = Color::okhsla(h, s, lightness, a);

            if let Some(ref mut on_input) = self.on_input {
                on_input(cx, data, color);
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
    style: ColorPickerStyle,
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

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        ColorPickerState {
            style: ColorPickerStyle::styled(self, cx.styles()),
            image: None,
            edit: None,
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);
        state.style.rebuild(self, cx);

        let (_, _, l, a) = self.color.to_okhsla();
        let (_, _, old_l, old_a) = old.color.to_okhsla();

        if (l - old_l).abs() > 1e-6 || (a - old_a).abs() > 1e-6 {
            state.image = None;
            cx.draw();
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
        state: &mut Self::State,
        _cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        space.fit(Size::all(state.style.size))
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, _data: &mut T) {
        let radius = cx.size().min_element() / 2.0;
        let wheel_radius = radius - state.style.slider_width;

        let (h, s, l, a) = self.color.to_okhsla();

        let image = state.image.get_or_insert_with(|| Self::wheel_image(l, a));

        cx.translated(cx.rect().center() - cx.rect().top_left(), |cx| {
            cx.hoverable(|cx| {
                // draw the sliders
                let lightness_angle = -Self::SLIDER_HALF + l * Self::SLIDER_ARC;
                let alpha_angle = Self::SLIDER_HALF - a * Self::SLIDER_ARC;

                cx.rotated(lightness_angle, |cx| {
                    cx.quad(
                        Rect::center_size(
                            Point::new(-wheel_radius, 0.0),
                            Size::new(
                                state.style.slider_width * 2.0,
                                state.style.slider_width * 1.5,
                            ),
                        ),
                        state.style.lightness_color,
                        state.style.slider_width / 2.0,
                        state.style.border_width / 2.0,
                        state.style.border_color,
                    );
                });

                cx.rotated(alpha_angle, |cx| {
                    cx.quad(
                        Rect::center_size(
                            Point::new(wheel_radius, 0.0),
                            Size::new(
                                state.style.slider_width * 2.0,
                                state.style.slider_width * 1.5,
                            ),
                        ),
                        state.style.alpha_color,
                        state.style.slider_width / 2.0,
                        state.style.border_width / 2.0,
                        state.style.border_color,
                    );
                });

                // draw the wheel
                cx.fill(
                    Curve::circle(Point::ZERO, wheel_radius + state.style.border_width),
                    FillRule::NonZero,
                    state.style.border_color,
                );

                let pattern = Pattern {
                    image: image.clone(),
                    transform: Affine::translate(cx.rect().top_left() - cx.rect().center())
                        * Affine::scale(Vector::from(cx.size() / image.size())),
                    opacity: 1.0,
                };

                cx.fill(
                    Curve::circle(Point::ZERO, wheel_radius),
                    FillRule::NonZero,
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
