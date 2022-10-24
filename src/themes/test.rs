use std::cell::Cell;

use crate::themes::Theme;

thread_local! {
  static THEME: Cell<Theme> = Cell::new(Theme::Light);
}

pub(crate) fn set_theme(theme: Theme) {
  THEME.with(|t| t.set(theme));
}

pub(crate) fn get_theme() -> Theme {
  THEME.with(|t| t.get())
}
