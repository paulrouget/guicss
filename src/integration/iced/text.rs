use std::borrow::Cow;

use iced::widget::text;
use iced_native::Length;

use crate::element::Element;
use crate::integration::iced::shared_rules::SharedRules;
use crate::integration::iced::{IdAndClasses, CSS};
use crate::properties::ComputedProperties;

/// 1. `CSS::text` constructor (with layout style)
/// 2. `text::StyleSheet` implementation (non-layout style)
/// 3. `ComputedProperties` to `text::Appearance`

impl CSS {
  pub fn text<'a, Renderer>(&self, content: impl Into<Cow<'a, str>>, def: IdAndClasses) -> iced::widget::Text<'a, Renderer>
  where Renderer: iced_native::text::Renderer<Theme = SharedRules> {
    let elt = Element::def("text", &def);
    let props = self.rules.compute(&elt);

    // FIXME:
    // size(self, size: u16) // Sets the size of the Text.
    // font(self, font: impl Into<Renderer::Font>) // Sets the Font of the Text.
    // fn width(self, width: Length)
    // fn height(self, height: Length)
    // fn horizontal_alignment(self, alignment: Horizontal) -> Self
    // fn vertical_alignment(self, alignment: Vertical) -> Self
    let mut text = iced::widget::Text::new(content);

    if let Some(size) = props.font_size {
      text = text.size(size as u16);
    }

    // FIXME: missing font

    let height = match props.height {
      None => Length::Shrink,
      Some(h) => Length::Units(h as u16),
    };

    let width = match props.width {
      None => Length::Shrink,
      Some(w) => Length::Units(w as u16),
    };

    text
      .width(width)
      .height(height)
      .horizontal_alignment(props.text_align.into())
      .vertical_alignment(props.vertical_align.into())
      .style(def)
  }
}

impl text::StyleSheet for SharedRules {
  type Style = IdAndClasses;

  fn appearance(&self, def: Self::Style) -> text::Appearance {
    let elt = Element::def("text", &def);
    self.compute(&elt).into()
  }
}

impl From<ComputedProperties> for text::Appearance {
  fn from(c: ComputedProperties) -> text::Appearance {
    text::Appearance {
      color: c.color.to_opt().map(|c| c.into()),
    }
  }
}
