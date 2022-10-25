#[derive(Debug)]
pub(crate) enum Event {
  Changed,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum SystemTheme {
  #[default]
  Light,
  Dark,
}

#[cfg(all(target_os = "macos", not(test)))]
#[path = "osx.rs"]
mod platform;

#[cfg(test)]
#[path = "test.rs"]
mod platform;

pub(crate) use platform::*;
