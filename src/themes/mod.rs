#[allow(dead_code)] // FIXME: to remove once we have non-osx implementation
#[derive(Debug)]
pub(crate) enum Event {
  Invalidated,
}

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

pub(crate) use platform::*;
