#[derive(Clone, Debug)]
pub(crate) enum Property {
  PaddingTop(f32),
  PaddingBottom(f32),
  PaddingLeft(f32),
  PaddingRight(f32),
  MarginLeft(f32),
  MarginTop(f32),
  MarginRight(f32),
  MarginBottom(f32),
}

#[derive(Clone, Debug)]
pub(crate) enum Importance {
  Important,
  NotImportant,
}

pub struct ComputedProperties {}
