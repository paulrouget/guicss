use iced::widget::text;
use iced::{application, Background, Vector};

use crate::element::{Element, PseudoClass};
use crate::integration::iced::shared_rules::SharedRules;
use crate::integration::iced::{IdAndClasses, CSS};
use crate::properties::ComputedProperties;

/// . ComputedProperties to application::Appearance

impl From<ComputedProperties> for application::Appearance {
  fn from(c: ComputedProperties) -> application::Appearance {
    application::Appearance {
      background_color: c.background_color.into(),
      text_color: c.color.into(),
    }
  }
}
