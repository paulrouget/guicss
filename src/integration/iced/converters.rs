use iced::widget::{button, text};
use iced::{application, Background, Vector};

use crate::properties::{Color, ComputedProperties, Sides};

impl From<Color> for iced::Color {
  fn from(c: Color) -> iced::Color {
    iced::Color {
      r: c.r as f32 / 255.0,
      g: c.g as f32 / 255.0,
      b: c.b as f32 / 255.0,
      a: c.a as f32 / 255.0,
    }
  }
}

impl From<Sides<f32>> for iced::Padding {
  fn from(s: Sides<f32>) -> iced::Padding {
    iced::Padding {
      top: s.top as u16,
      right: s.right as u16,
      bottom: s.bottom as u16,
      left: s.left as u16,
    }
  }
}

impl From<ComputedProperties> for application::Appearance {
  fn from(c: ComputedProperties) -> application::Appearance {
    application::Appearance {
      background_color: c.background_color.into(),
      text_color: c.color.into(),
    }
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

impl From<ComputedProperties> for text::Appearance {
  fn from(c: ComputedProperties) -> text::Appearance {
    text::Appearance {
      color: c.color.to_opt().map(|c| c.into()),
    }
  }
}
