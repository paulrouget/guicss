use crate::properties::{Color, Sides};

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
