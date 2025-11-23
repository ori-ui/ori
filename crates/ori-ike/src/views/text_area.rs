use ike::{
    BuildCx, Color, FontStretch, FontStyle, FontWeight, Paragraph, TextAlign, TextStyle, TextWrap,
    widgets::{NewlineBehaviour, SubmitBehaviour},
};
use ori::{AsyncContext, ProviderContext, Proxy};

use crate::{Context, Palette, views::TextTheme};

pub fn text_area<T>() -> TextArea<T> {
    TextArea::new()
}

pub struct TextArea<T> {
    text:         Option<String>,
    font_size:    Option<f32>,
    font_family:  Option<String>,
    font_weight:  Option<FontWeight>,
    font_stretch: Option<FontStretch>,
    font_style:   Option<FontStyle>,
    line_height:  Option<f32>,
    align:        Option<TextAlign>,
    wrap:         Option<TextWrap>,
    color:        Option<Color>,

    cursor_color:      Option<Color>,
    selection_color:   Option<Color>,
    blink_rate:        f32,
    newline_behaviour: NewlineBehaviour,
    submit_behaviour:  SubmitBehaviour,

    #[allow(clippy::type_complexity)]
    on_change: Box<dyn FnMut(&mut T, String) -> ori::Action>,
    #[allow(clippy::type_complexity)]
    on_submit: Box<dyn FnMut(&mut T, String) -> ori::Action>,
}

impl<T> Default for TextArea<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TextArea<T> {
    pub fn new() -> Self {
        Self {
            text:         None,
            font_size:    None,
            font_family:  None,
            font_weight:  None,
            font_stretch: None,
            font_style:   None,
            line_height:  None,
            align:        None,
            wrap:         None,
            color:        None,

            cursor_color:      None,
            selection_color:   None,
            blink_rate:        5.0,
            newline_behaviour: NewlineBehaviour::Enter,
            submit_behaviour:  SubmitBehaviour::FocusNext,

            on_change: Box::new(|_, _| ori::Action::new()),
            on_submit: Box::new(|_, _| ori::Action::new()),
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
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

    pub fn selection_color(mut self, color: Color) -> Self {
        self.selection_color = Some(color);
        self
    }

    pub fn cursor_color(mut self, color: Color) -> Self {
        self.cursor_color = Some(color);
        self
    }

    pub fn blink_rate(mut self, rate: f32) -> Self {
        self.blink_rate = rate;
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

    pub fn on_change<A>(mut self, mut on_change: impl FnMut(&mut T, String) -> A + 'static) -> Self
    where
        A: ori::IntoAction,
    {
        self.on_change = Box::new(move |data, text| on_change(data, text).into_action());
        self
    }

    pub fn on_submit<A>(mut self, mut on_submit: impl FnMut(&mut T, String) -> A + 'static) -> Self
    where
        A: ori::IntoAction,
    {
        self.on_submit = Box::new(move |data, text| on_submit(data, text).into_action());
        self
    }
}

impl<T> TextArea<T> {
    fn build_paragraph(&self, text: &str, palette: &Palette, theme: &TextTheme) -> Paragraph {
        let style = TextStyle {
            font_size:    self.font_size.unwrap_or(theme.font_size),
            font_weight:  self.font_weight.unwrap_or(theme.font_weight),
            font_stretch: self.font_stretch.unwrap_or(theme.font_stretch),
            font_style:   self.font_style.unwrap_or(theme.font_style),

            font_family: self
                .font_family
                .clone()
                .unwrap_or_else(|| theme.font_family.clone().into_owned()),

            color: self
                .color
                .unwrap_or_else(|| theme.color.unwrap_or(palette.contrast)),
        };

        let mut paragraph = Paragraph::new(
            self.line_height.unwrap_or(theme.line_height),
            self.align.unwrap_or(theme.align),
            self.wrap.unwrap_or(theme.wrap),
        );

        paragraph.push(text, style);
        paragraph
    }
}

enum TextAreaEvent {
    Change(String),
    Submit(String),
}

impl<T> ori::ViewMarker for TextArea<T> {}
impl<T> ori::View<Context, T> for TextArea<T> {
    type Element = ike::WidgetId<ike::widgets::TextArea>;
    type State = ori::ViewId;

    fn build(&mut self, cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();
        let theme = cx.get_context::<TextTheme>().cloned().unwrap_or_default();
        let proxy = cx.proxy();
        let id = ori::ViewId::next();

        let text = self.text.as_deref().unwrap_or("");
        let paragraph = self.build_paragraph(text, &palette, &theme);

        let mut widget = ike::widgets::TextArea::new(cx, paragraph, true);

        let cursor_color = self.cursor_color.unwrap_or(palette.contrast);
        let selection_color = self.selection_color.unwrap_or(palette.info);

        ike::widgets::TextArea::set_cursor_color(&mut widget, cursor_color);
        ike::widgets::TextArea::set_selection_color(&mut widget, selection_color);
        ike::widgets::TextArea::set_blink_rate(&mut widget, self.blink_rate);
        ike::widgets::TextArea::set_newline_behaviour(&mut widget, self.newline_behaviour);

        ike::widgets::TextArea::set_on_change(&mut widget, {
            let proxy = proxy.clone();

            move |text| {
                proxy.event(ori::Event::new(
                    TextAreaEvent::Change(text.into()),
                    id,
                ))
            }
        });

        ike::widgets::TextArea::set_on_submit(&mut widget, {
            let proxy = proxy.clone();

            move |text| {
                proxy.event(ori::Event::new(
                    TextAreaEvent::Submit(text.into()),
                    id,
                ))
            }
        });

        (widget.id(), id)
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        _state: &mut Self::State,
        cx: &mut Context,
        _data: &mut T,
        old: &mut Self,
    ) {
        let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();
        let theme = cx.get_context::<TextTheme>().cloned().unwrap_or_default();

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
            let text = self.text.as_deref().unwrap_or_else(|| widget.text());

            let paragraph = self.build_paragraph(text, &palette, &theme);
            ike::widgets::TextArea::set_text(&mut widget, paragraph);
        }

        if self.cursor_color != old.cursor_color {
            let cursor_color = self.cursor_color.unwrap_or(palette.contrast);
            ike::widgets::TextArea::set_cursor_color(&mut widget, cursor_color);
        }

        if self.selection_color != old.selection_color {
            let selection_color = self.selection_color.unwrap_or(palette.info);
            ike::widgets::TextArea::set_selection_color(&mut widget, selection_color);
        }

        if self.blink_rate != old.blink_rate {
            ike::widgets::TextArea::set_blink_rate(&mut widget, self.blink_rate);
        }

        if self.newline_behaviour != old.newline_behaviour {
            ike::widgets::TextArea::set_newline_behaviour(&mut widget, self.newline_behaviour);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        _state: Self::State,
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
            Some(TextAreaEvent::Change(text)) => (self.on_change)(data, text),
            Some(TextAreaEvent::Submit(text)) => (self.on_submit)(data, text),
            None => ori::Action::new(),
        }
    }
}
