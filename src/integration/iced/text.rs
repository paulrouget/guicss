use std::borrow::Cow;

use iced::widget::text;
use iced::{application, Background, Vector};

use crate::element::{Element, PseudoClass};
use crate::integration::iced::shared_rules::SharedRules;
use crate::integration::iced::{IdAndClasses, CSS};
use crate::properties::ComputedProperties;

/// 1. CSS::text constructor (with layout style)
/// 2. text::StyleSheet implementation (non-layout style)
/// 3. ComputedProperties to text::Appearance

impl CSS {
  pub fn text<'a, Renderer>(&self, content: impl Into<Cow<'a, str>>, def: IdAndClasses) -> iced::widget::Text<'a, Renderer>
  where Renderer: iced_native::text::Renderer<Theme = SharedRules> {
    use iced_native::Length;

    let elt = Element::def("text", &def);
    let props = self.rules.compute(&elt);

    // FIXME:
    // size(self, size: u16) // Sets the size of the Text.
    // font(self, font: impl Into<Renderer::Font>) // Sets the Font of the Text.
    // fn width(self, width: Length)
    // fn height(self, height: Length)
    // fn horizontal_alignment(self, alignment: Horizontal) -> Self
    // fn vertical_alignment(self, alignment: Vertical) -> Self
    iced::widget::Text::new(content)
        // .size(size)
        // .font(font)
        // .width(width)
        // .height(height)
        // .horizontal_alignment(align_h)
        // .vertical_alignment(align_v)
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
