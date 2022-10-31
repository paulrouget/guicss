use iced::widget::button;
use iced::{Background, Vector};
use iced_native::Length;

use crate::element::{Element, PseudoClass};
use crate::integration::iced::shared_rules::SharedRules;
use crate::integration::iced::{IdAndClasses, CSS};
use crate::properties::ComputedProperties;

/// 1. `CSS::button` constructor (with layout style)
/// 2. `button::StyleSheet` implementation (non-layout style)
/// 3. `ComputedProperties` to `button::Appearance`

impl CSS {
  pub fn button<'a, Message, Renderer>(
    &self,
    content: impl Into<iced::Element<'a, Message, Renderer>>,
    def: IdAndClasses,
  ) -> iced::widget::Button<'a, Message, Renderer>
  where
    Renderer: iced_native::renderer::Renderer<Theme = SharedRules>,
  {
    let elt = Element::def("button", &def);
    let props = self.rules.compute(&elt);
    let width = match (props.flex_grow as u16, props.flex_basis, props.width) {
      (0, None, None) => Length::Shrink,
      (0, None, Some(w)) => Length::Units(w as u16),
      (0, Some(w), _) => Length::Units(w as u16),
      (grow, _, _) => Length::FillPortion(grow),
    };
    let height = match props.height {
      None => Length::Shrink,
      Some(h) => Length::Units(h as u16),
    };
    iced::widget::Button::new(content).padding(props.padding).width(width).height(height).style(def)
  }
}

impl button::StyleSheet for SharedRules {
  type Style = IdAndClasses;

  fn active(&self, def: Self::Style) -> button::Appearance {
    let elt = Element::def("button", &def);
    self.compute(&elt).into()
  }

  fn hovered(&self, def: Self::Style) -> button::Appearance {
    let elt = Element::def("button", &def).pseudo_class(PseudoClass::Hover);
    self.compute(&elt).into()
  }

  fn pressed(&self, def: Self::Style) -> button::Appearance {
    let elt = Element::def("button", &def).pseudo_class(PseudoClass::Active);
    self.compute(&elt).into()
  }

  fn disabled(&self, def: Self::Style) -> button::Appearance {
    let elt = Element::def("button", &def).pseudo_class(PseudoClass::Disabled);
    self.compute(&elt).into()
  }
}

impl From<ComputedProperties> for button::Appearance {
  fn from(c: ComputedProperties) -> button::Appearance {
    button::Appearance {
      background: c.background_color.to_opt().map(|c| Background::Color(c.into())),
      border_radius: c.border_radius.nw.0,
      border_width: c.border.top.width,
      border_color: c.border.top.color.into(),
      text_color: c.color.into(),
      // FIXME: unmapped
      shadow_offset: Vector::default(),
    }
  }
}
