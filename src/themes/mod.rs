#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum Theme {
  #[default]
  Light,
  #[allow(dead_code)]
  Dark,
}

#[cfg(all(target_os = "macos", not(test)))]
#[path = "osx.rs"]
mod platform;

#[cfg(test)]
#[path = "test.rs"]
mod platform;

#[cfg(all(not(target_os = "macos"), not(test)))]
mod platform {
  use super::Theme;
  pub(crate) fn get_theme() -> Theme {
    Theme::default()
  }
}

pub(crate) use platform::*;
