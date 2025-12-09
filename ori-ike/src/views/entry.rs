use ike::{
    BorderWidth, BuildCx, Color, CornerRadius, FontStretch, FontStyle, FontWeight, Padding,
    Paragraph, TextAlign, TextStyle, TextWrap,
    widgets::{NewlineBehaviour, SubmitBehaviour},
};
use ori::{AsyncContext, ProviderContext, Proxy};

use crate::{Context, Palette, views::TextTheme};

#[derive(Clone, Debug)]
pub struct EntryTheme {
    pub font_size:         Option<f32>,
    pub font_family:       Option<String>,
    pub font_weight:       Option<FontWeight>,
    pub font_stretch:      Option<FontStretch>,
    pub font_style:        Option<FontStyle>,
    pub line_height:       Option<f32>,
    pub align:             Option<TextAlign>,
    pub wrap:              Option<TextWrap>,
    pub color:             Option<Color>,
    pub placeholder_color: Option<Color>,
    pub min_width:         f32,
    pub max_width:         f32,
    pub padding:           Padding,
    pub border_width:      BorderWidth,
    pub corner_radius:     CornerRadius,
    pub background_color:  Option<Color>,
    pub border_color:      Option<Color>,
    pub focus_color:       Option<Color>,
    pub cursor_color:      Option<Color>,
    pub selection_color:   Option<Color>,
    pub blink_rate:        f32,
}

impl Default for EntryTheme {
    fn default() -> Self {
        Self {
            font_size:         None,
            font_family:       None,
            font_weight:       None,
            font_stretch:      None,
            font_style:        None,
            line_height:       None,
            align:             None,
            wrap:              None,
            color:             None,
            placeholder_color: None,
            min_width:         100.0,
            max_width:         f32::INFINITY,
            padding:           Padding::all(8.0),
            border_width:      BorderWidth::all(1.0),
            corner_radius:     CornerRadius::all(8.0),
            background_color:  None,
            border_color:      None,
            focus_color:       None,
            cursor_color:      None,
            selection_color:   None,
            blink_rate:        5.0,
        }
    }
}

pub fn entry<T>() -> Entry<T> {
    Entry::new()
}

pub struct Entry<T> {
    text:              Option<String>,
    placeholder:       String,
    font_size:         Option<f32>,
    font_family:       Option<String>,
    font_weight:       Option<FontWeight>,
    font_stretch:      Option<FontStretch>,
    font_style:        Option<FontStyle>,
    line_height:       Option<f32>,
    align:             Option<TextAlign>,
    wrap:              Option<TextWrap>,
    color:             Option<Color>,
    placeholder_color: Option<Color>,
    min_width:         Option<f32>,
    max_width:         Option<f32>,
    padding:           Option<Padding>,
    border_width:      Option<BorderWidth>,
    corner_radius:     Option<CornerRadius>,
    background_color:  Option<Color>,
    border_color:      Option<Color>,
    focus_color:       Option<Color>,
    cursor_color:      Option<Color>,
    selection_color:   Option<Color>,
    blink_rate:        Option<f32>,
    newline_behaviour: NewlineBehaviour,
    submit_behaviour:  SubmitBehaviour,

    #[allow(clippy::type_complexity)]
    on_change: Box<dyn FnMut(&mut T, String) -> ori::Action>,
    #[allow(clippy::type_complexity)]
    on_submit: Box<dyn FnMut(&mut T, String) -> ori::Action>,
}

impl<T> Default for Entry<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Entry<T> {
    pub fn new() -> Self {
        Self {
            text:              None,
            placeholder:       String::from("..."),
            font_size:         None,
            font_family:       None,
            font_weight:       None,
            font_stretch:      None,
            font_style:        None,
            line_height:       None,
            align:             None,
            wrap:              None,
            color:             None,
            placeholder_color: None,
            min_width:         None,
            max_width:         None,
            padding:           None,
            border_width:      None,
            corner_radius:     None,
            background_color:  None,
            border_color:      None,
            focus_color:       None,
            cursor_color:      None,
            selection_color:   None,
            blink_rate:        None,
            newline_behaviour: NewlineBehaviour::Never,
            submit_behaviour:  SubmitBehaviour::default(),

            on_change: Box::new(|_, _| ori::Action::new()),
            on_submit: Box::new(|_, _| ori::Action::new()),
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = Some(font_size);
        self
    }

    pub fn font_family(mut self, font_family: impl ToString) -> Self {
        self.font_family = Some(font_family.to_string());
        self
    }

    pub fn font_weight(mut self, font_weight: FontWeight) -> Self {
        self.font_weight = Some(font_weight);
        self
    }

    pub fn font_stretch(mut self, font_stretch: FontStretch) -> Self {
        self.font_stretch = Some(font_stretch);
        self
    }

    pub fn font_style(mut self, font_style: FontStyle) -> Self {
        self.font_style = Some(font_style);
        self
    }

    pub fn line_height(mut self, line_height: f32) -> Self {
        self.line_height = Some(line_height);
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = Some(align);
        self
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = Some(wrap);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn placeholder_color(mut self, color: Color) -> Self {
        self.placeholder_color = Some(color);
        self
    }

    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn border_width(mut self, border_width: impl Into<BorderWidth>) -> Self {
        self.border_width = Some(border_width.into());
        self
    }

    pub fn corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.corner_radius = Some(corner_radius.into());
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn focus_color(mut self, color: Color) -> Self {
        self.focus_color = Some(color);
        self
    }

    pub fn selection_color(mut self, color: Color) -> Self {
        self.selection_color = Some(color);
        self
    }

    pub fn cursor_color(mut self, color: Color) -> Self {
        self.cursor_color = Some(color);
        self
    }

    pub fn blink_rate(mut self, rate: f32) -> Self {
        self.blink_rate = Some(rate);
        self
    }

    pub fn newline_behaviour(mut self, behaviour: NewlineBehaviour) -> Self {
        self.newline_behaviour = behaviour;
        self
    }

    pub fn submit_behaviour(mut self, behaviour: SubmitBehaviour) -> Self {
        self.submit_behaviour = behaviour;
        self
    }

    pub fn on_change<A, I>(
        mut self,
        mut on_change: impl FnMut(&mut T, String) -> A + 'static,
    ) -> Self
    where
        A: ori::IntoAction<I>,
    {
        self.on_change = Box::new(move |data, text| on_change(data, text).into_action());
        self
    }

    pub fn on_submit<A, I>(
        mut self,
        mut on_submit: impl FnMut(&mut T, String) -> A + 'static,
    ) -> Self
    where
        A: ori::IntoAction<I>,
    {
        self.on_submit = Box::new(move |data, text| on_submit(data, text).into_action());
        self
    }
}

impl<T> Entry<T> {
    fn build_paragraph(
        &self,
        text: &str,
        color: Color,
        text_theme: &TextTheme,
        entry_theme: &EntryTheme,
    ) -> Paragraph {
        let style = TextStyle {
            font_size: self
                .font_size
                .unwrap_or_else(|| entry_theme.font_size.unwrap_or(text_theme.font_size)),

            font_weight: self
                .font_weight
                .unwrap_or_else(|| entry_theme.font_weight.unwrap_or(text_theme.font_weight)),

            font_stretch: self
                .font_stretch
                .unwrap_or_else(|| entry_theme.font_stretch.unwrap_or(text_theme.font_stretch)),

            font_style: self
                .font_style
                .unwrap_or_else(|| entry_theme.font_style.unwrap_or(text_theme.font_style)),

            font_family: self.font_family.clone().unwrap_or_else(|| {
                entry_theme
                    .font_family
                    .clone()
                    .unwrap_or_else(|| text_theme.font_family.clone().into_owned())
            }),

            color,
        };

        let mut paragraph = Paragraph::new(
            self.line_height
                .unwrap_or_else(|| entry_theme.line_height.unwrap_or(text_theme.line_height)),
            self.align
                .unwrap_or_else(|| entry_theme.align.unwrap_or(text_theme.align)),
            self.wrap
                .unwrap_or_else(|| entry_theme.wrap.unwrap_or(text_theme.wrap)),
        );

        paragraph.push(text, style);
        paragraph
    }

    fn get_color(&self, palette: &Palette, text_theme: &TextTheme, theme: &EntryTheme) -> Color {
        self.color.unwrap_or_else(|| {
            theme
                .color
                .unwrap_or_else(|| text_theme.color.unwrap_or(palette.contrast))
        })
    }

    fn get_placeholder_color(&self, palette: &Palette, theme: &EntryTheme) -> Color {
        self.placeholder_color.unwrap_or_else(|| {
            theme
                .placeholder_color
                .unwrap_or_else(|| palette.contrast_low(0))
        })
    }

    fn get_background_color(&self, palette: &Palette, theme: &EntryTheme) -> Color {
        self.background_color.unwrap_or_else(|| {
            theme
                .background_color
                .unwrap_or_else(|| palette.surface(-1))
        })
    }

    fn get_border_color(&self, palette: &Palette, theme: &EntryTheme) -> Color {
        self.border_color
            .unwrap_or_else(|| theme.border_color.unwrap_or(palette.outline))
    }

    fn get_focus_color(&self, palette: &Palette, theme: &EntryTheme) -> Color {
        self.focus_color
            .unwrap_or_else(|| theme.focus_color.unwrap_or(palette.info))
    }

    fn get_cursor_color(&self, palette: &Palette, theme: &EntryTheme) -> Color {
        self.cursor_color
            .unwrap_or_else(|| theme.cursor_color.unwrap_or(palette.contrast))
    }

    fn get_selection_color(&self, palette: &Palette, theme: &EntryTheme) -> Color {
        self.selection_color
            .unwrap_or_else(|| theme.selection_color.unwrap_or(palette.info))
    }
}

enum EntryEvent {
    Change(String),
    Submit(String),
}

impl<T> ori::ViewMarker for Entry<T> {}
impl<T> ori::View<Context, T> for Entry<T> {
    type Element = ike::WidgetId<ike::widgets::Entry>;
    type State = ori::ViewId;

    fn build(&mut self, cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();
        let text_theme = cx.get_context::<TextTheme>().cloned().unwrap_or_default();
        let theme = cx.get_context::<EntryTheme>().cloned().unwrap_or_default();
        let proxy = cx.proxy();
        let id = ori::ViewId::next();

        let color = self.get_color(&palette, &text_theme, &theme);
        let placeholder_color = self.get_placeholder_color(&palette, &theme);

        let text = self.text.as_deref().unwrap_or("");
        let paragraph = self.build_paragraph(text, color, &text_theme, &theme);
        let placeholder = self.build_paragraph(
            &self.placeholder,
            placeholder_color,
            &text_theme,
            &theme,
        );

        let mut widget = ike::widgets::Entry::new(cx, paragraph);

        let min_width = self.min_width.unwrap_or(theme.min_width);
        let max_width = self.max_width.unwrap_or(theme.max_width);
        let padding = self.padding.unwrap_or(theme.padding);
        let border_width = self.border_width.unwrap_or(theme.border_width);
        let corner_radius = self.corner_radius.unwrap_or(theme.corner_radius);
        let background_color = self.get_background_color(&palette, &theme);
        let border_color = self.get_border_color(&palette, &theme);
        let focus_color = self.get_focus_color(&palette, &theme);
        let cursor_color = self.get_cursor_color(&palette, &theme);
        let selection_color = self.get_selection_color(&palette, &theme);
        let blink_rate = self.blink_rate.unwrap_or(theme.blink_rate);

        ike::widgets::Entry::set_placeholder(&mut widget, placeholder);
        ike::widgets::Entry::set_min_width(&mut widget, min_width);
        ike::widgets::Entry::set_max_width(&mut widget, max_width);
        ike::widgets::Entry::set_padding(&mut widget, padding);
        ike::widgets::Entry::set_border_width(&mut widget, border_width);
        ike::widgets::Entry::set_corner_radius(&mut widget, corner_radius);
        ike::widgets::Entry::set_background_color(&mut widget, background_color);
        ike::widgets::Entry::set_border_color(&mut widget, border_color);
        ike::widgets::Entry::set_focus_color(&mut widget, focus_color);
        ike::widgets::Entry::set_cursor_color(&mut widget, cursor_color);
        ike::widgets::Entry::set_selection_color(&mut widget, selection_color);
        ike::widgets::Entry::set_blink_rate(&mut widget, blink_rate);
        ike::widgets::Entry::set_newline_behaviour(&mut widget, self.newline_behaviour);
        ike::widgets::Entry::set_submit_behaviour(&mut widget, self.submit_behaviour);

        ike::widgets::Entry::set_on_change(&mut widget, {
            let proxy = proxy.clone();

            move |text| {
                proxy.event(ori::Event::new(
                    EntryEvent::Change(text.into()),
                    id,
                ))
            }
        });

        ike::widgets::Entry::set_on_submit(&mut widget, {
            let proxy = proxy.clone();

            move |text| {
                proxy.event(ori::Event::new(
                    EntryEvent::Submit(text.into()),
                    id,
                ))
            }
        });

        (widget.id(), id)
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        _id: &mut Self::State,
        cx: &mut Context,
        _data: &mut T,
        old: &mut Self,
    ) {
        let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();
        let text_theme = cx.get_context::<TextTheme>().cloned().unwrap_or_default();
        let theme = cx.get_context::<EntryTheme>().cloned().unwrap_or_default();

        let mut widget = cx.get_mut(*element);

        if self.text != old.text
            || self.font_size != old.font_size
            || self.font_family != old.font_family
            || self.font_weight != old.font_weight
            || self.font_stretch != old.font_stretch
            || self.font_style != old.font_style
            || self.line_height != old.line_height
            || self.align != old.align
            || self.wrap != old.wrap
            || self.color != old.color
        {
            let text = match self.text {
                Some(ref text) => text.clone(),
                None => ike::widgets::Entry::text_area(&widget.as_ref())
                    .text()
                    .to_owned(),
            };

            let color = self.get_color(&palette, &text_theme, &theme);
            let paragraph = self.build_paragraph(&text, color, &text_theme, &theme);
            ike::widgets::Entry::set_text(&mut widget, paragraph);
        }

        if self.placeholder != old.placeholder || self.placeholder_color != old.placeholder_color {
            let placeholder_color = self.get_placeholder_color(&palette, &theme);
            let placeholder = self.build_paragraph(
                &self.placeholder,
                placeholder_color,
                &text_theme,
                &theme,
            );
            ike::widgets::Entry::set_placeholder(&mut widget, placeholder);
        }

        if self.min_width != old.min_width {
            let min_width = self.min_width.unwrap_or(theme.min_width);
            ike::widgets::Entry::set_min_width(&mut widget, min_width);
        }

        if self.max_width != old.max_width {
            let max_width = self.max_width.unwrap_or(theme.max_width);
            ike::widgets::Entry::set_max_width(&mut widget, max_width);
        }

        if self.padding != old.padding {
            let padding = self.padding.unwrap_or(theme.padding);
            ike::widgets::Entry::set_padding(&mut widget, padding);
        }

        if self.border_width != old.border_width {
            let border_width = self.border_width.unwrap_or(theme.border_width);
            ike::widgets::Entry::set_border_width(&mut widget, border_width);
        }

        if self.corner_radius != old.corner_radius {
            let corner_radius = self.corner_radius.unwrap_or(theme.corner_radius);
            ike::widgets::Entry::set_corner_radius(&mut widget, corner_radius);
        }

        if self.background_color != old.background_color {
            let background_color = self.get_background_color(&palette, &theme);
            ike::widgets::Entry::set_background_color(&mut widget, background_color);
        }

        if self.border_color != old.border_color {
            let border_color = self.get_border_color(&palette, &theme);
            ike::widgets::Entry::set_border_color(&mut widget, border_color);
        }

        if self.focus_color != old.focus_color {
            let focus_color = self.get_focus_color(&palette, &theme);
            ike::widgets::Entry::set_focus_color(&mut widget, focus_color);
        }

        if self.cursor_color != old.cursor_color {
            let cursor_color = self.get_cursor_color(&palette, &theme);
            ike::widgets::Entry::set_cursor_color(&mut widget, cursor_color);
        }

        if self.selection_color != old.selection_color {
            let selection_color = self.get_selection_color(&palette, &theme);
            ike::widgets::Entry::set_selection_color(&mut widget, selection_color);
        }

        if self.blink_rate != old.blink_rate {
            let blink_rate = self.blink_rate.unwrap_or(theme.blink_rate);
            ike::widgets::Entry::set_blink_rate(&mut widget, blink_rate);
        }

        if self.newline_behaviour != old.newline_behaviour {
            ike::widgets::Entry::set_newline_behaviour(&mut widget, self.newline_behaviour);
        }

        if self.submit_behaviour != old.submit_behaviour {
            ike::widgets::Entry::set_submit_behaviour(&mut widget, self.submit_behaviour);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        _id: Self::State,
        cx: &mut Context,
        _data: &mut T,
    ) {
        cx.remove(element);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        id: &mut Self::State,
        _cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        match event.take_targeted(*id) {
            Some(EntryEvent::Change(text)) => (self.on_change)(data, text),
            Some(EntryEvent::Submit(text)) => (self.on_submit)(data, text),
            None => ori::Action::new(),
        }
    }
}
