use crate::properties::{Align, Color, Sides};

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

impl From<Align> for iced_native::alignment::Horizontal {
  fn from(s: Align) -> Self {
    match s {
      Align::Start => Self::Left,
      Align::Justify => Self::Left,
      Align::End => Self::Right,
      Align::Center => Self::Center,
    }
  }
}

impl From<Align> for iced_native::alignment::Vertical {
  fn from(s: Align) -> Self {
    match s {
      Align::Start => Self::Top,
      Align::Justify => Self::Top,
      Align::End => Self::Bottom,
      Align::Center => Self::Center,
    }
  }
}
