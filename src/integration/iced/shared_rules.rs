use std::sync::Arc;

use iced::application;
use iced::widget::{button, text};
use parking_lot::RwLock;

use crate::element::{Element, PseudoClass};
use crate::integration::iced::IdAndClasses;
use crate::parser::Rules;
use crate::properties::ComputedProperties;

#[derive(Clone)]
pub struct SharedRules(Arc<RwLock<Rules>>);

impl SharedRules {
  pub(crate) fn new(rules: Rules) -> SharedRules {
    SharedRules(Arc::new(RwLock::new(rules)))
  }

  pub(crate) fn update(&self, rules: Rules) {
    *self.0.write() = rules;
  }

  fn compute(&self, elt: &Element<'_>) -> ComputedProperties {
    self.0.read().compute(elt)
  }
}

impl Default for SharedRules {
  fn default() -> Self {
    panic!("Stylesheet has to be initalized");
  }
}

impl application::StyleSheet for SharedRules {
  type Style = ();

  fn appearance(&self, _: Self::Style) -> application::Appearance {
    let elt = Element::root();
    self.compute(&elt).into()
  }
}

impl button::StyleSheet for SharedRules {
  type Style = IdAndClasses;

  fn active(&self, def: Self::Style) -> button::Appearance {
    let mut elt = Element::named("button");
    elt.set_id_and_classes(&def);
    self.compute(&elt).into()
  }

  fn hovered(&self, def: Self::Style) -> button::Appearance {
    let mut elt = Element::named("button").pseudo_class(PseudoClass::Hover);
    elt.set_id_and_classes(&def);
    self.compute(&elt).into()
  }

  fn pressed(&self, def: Self::Style) -> button::Appearance {
    let mut elt = Element::named("button").pseudo_class(PseudoClass::Active);
    elt.set_id_and_classes(&def);
    self.compute(&elt).into()
  }

  fn disabled(&self, def: Self::Style) -> button::Appearance {
    let mut elt = Element::named("button").pseudo_class(PseudoClass::Disabled);
    elt.set_id_and_classes(&def);
    self.compute(&elt).into()
  }
}

impl text::StyleSheet for SharedRules {
  type Style = IdAndClasses;

  fn appearance(&self, def: Self::Style) -> text::Appearance {
    let mut elt = Element::named("text");
    elt.set_id_and_classes(&def);
    self.compute(&elt).into()
  }
}
