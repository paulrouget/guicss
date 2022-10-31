use std::sync::Arc;

use iced::application;
use parking_lot::RwLock;

use crate::element::Element;
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

  pub(crate) fn compute(&self, elt: &Element<'_>) -> ComputedProperties {
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
