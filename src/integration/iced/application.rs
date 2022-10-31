use iced::application;

use crate::properties::ComputedProperties;

impl From<ComputedProperties> for application::Appearance {
  fn from(c: ComputedProperties) -> application::Appearance {
    application::Appearance {
      background_color: c.background_color.into(),
      text_color: c.color.into(),
    }
  }
}
