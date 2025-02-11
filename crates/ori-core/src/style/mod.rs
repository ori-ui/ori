//! Styling and theming.
//!
//! # Example
//! The below example demonstrates how a view with styling can be implemented.
//!
//! ```rust
//! # use ori_core::{
//! #    style::{Style, Styles, Stylable, StyleBuilder, Theme},
//! #    views::ButtonStyle,
//! #    canvas::Color,
//! #    layout::{Space, Size, Padding},
//! #    view::View,
//! #    event::Event,
//! #    context::{BuildCx, RebuildCx, EventCx, LayoutCx, DrawCx},
//! #    rebuild::Rebuild,
//! #    text::{Paragraph, TextAlign, TextWrap},
//! # };
//! #[derive(Clone, Rebuild)]
//! struct MyStyle {
//!     // we use `rebuild` to draw the view when `my_color` changes
//!     #[rebuild(draw)]
//!     my_color: Color,
//!
//!     // we use `rebuild` to layout the view when `my_padding` changes
//!     #[rebuild(layout)]
//!     my_padding: Padding,
//! }
//!
//! // `Style` must be implemented for use in `Styles`
//! impl Style for MyStyle {
//!     fn default_style() -> StyleBuilder<Self> {
//!         StyleBuilder::new(|theme: &Theme, button: &ButtonStyle| MyStyle {
//!             my_color: theme.accent,
//!             my_padding: button.padding,
//!         })
//!     }
//! }
//!
//! #[derive(Rebuild)]
//! struct MyView {
//!     // we use this field to override the color of the style
//!     //
//!     // note that this field doesn't have the `rebuild` attribute, this is
//!     // because `MyStyle` already has a `rebuild` attribute for `my_color`
//!     // and we only care about when the style changes
//!     my_color: Option<Color>,
//!
//!     // this field is not related to the style
//!     #[rebuild(layout)]
//!     non_styled_field: String,
//! }
//!
//! impl Stylable for MyView {
//!     type Style = MyStyle;
//!
//!     fn style(&self, base: &Self::Style) -> Self::Style {
//!         MyStyle {
//!             my_color: self.my_color.unwrap_or(base.my_color),
//!             
//!             // generally it's preferred to be able to override every field of the style
//!             // even though it's not required, the current implementation of `MyView` doesn't
//!             // allow overriding `my_padding`
//!             ..base.clone()
//!         }
//!     }
//! }
//!
//! impl<T> View<T> for MyView {
//!     // we need to store the style in the `State` of our `View` implementation
//!     type State = MyStyle;
//!
//!     fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
//!         // we build the `style` in the `build` method
//!         self.style(cx.style())
//!     }
//!
//!     fn rebuild(
//!         &mut self,
//!         state: &mut Self::State,
//!         cx: &mut RebuildCx,
//!         _data: &mut T,
//!         _old: &Self,
//!     ) {
//!         // we rebuild the `style` in the `rebuild` method
//!         self.rebuild_style(cx, state);
//!     }
//!
//!     fn event(
//!         &mut self,
//!         _state: &mut Self::State,
//!         _cx: &mut EventCx,
//!         _data: &mut T,
//!         _event: &Event,
//!     ) -> bool {
//!         // we don't care about events in this example
//!         false
//!     }
//!     
//!     fn layout(
//!         &mut self,
//!         state: &mut Self::State,
//!         cx: &mut LayoutCx,
//!         _data: &mut T,
//!         space: Space,
//!     ) -> Size {
//!         // we create a paragraph with the non-styled field
//!         let paragraph = Paragraph::new(1.0, TextAlign::Start, TextWrap::Word)
//!             .with_text(&self.non_styled_field, Default::default());
//!
//!         // we calculate the size that the padding will take
//!         let pad_size = state.my_padding.size();
//!
//!         // we calculate the maximum width that the paragraph can take
//!         let max_width = space.max.width - pad_size.width;
//!
//!         // we measure the paragraph
//!         let size = cx.measure_paragraph(&paragraph, max_width);
//!
//!         // we constrain the size of the view to fit the space given
//!         space.fit(size + pad_size)
//!     }
//!     
//!     fn draw(
//!         &mut self,
//!         state: &mut Self::State,
//!         cx: &mut DrawCx,
//!         _data: &mut T,
//!     ) {
//!         // we create a paragraph with the non-styled field
//!         let paragraph = Paragraph::new(1.0, TextAlign::Start, TextWrap::Word)
//!             .with_text(&self.non_styled_field, Default::default());
//!
//!         // we draw the paragraph offset by the padding
//!         cx.paragraph(&paragraph, cx.rect() + state.my_padding.offset());
//!     }
//! }
//! ```

mod styles;
mod theme;

pub use styles::*;
pub use theme::*;
