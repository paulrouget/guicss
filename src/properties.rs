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

impl Importance {
  pub fn is_important(&self) -> bool {
    matches!(self, Importance::Important)
  }
}

#[derive(Debug, Default)]
pub struct ComputedProperties {
  pub padding_top: f32,
}

impl ComputedProperties {
  pub(crate) fn import<'i, T: IntoIterator<Item = &'i Property>>(&mut self, props: T) {
    for prop in props.into_iter() {
      match prop {
        Property::PaddingTop(x) => self.padding_top = *x,
        _ => { /* FIXME */ }
      }
    }
  }
}
