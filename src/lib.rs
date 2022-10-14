//! `GuiCss` is a CSS parser designed for Rust Desktop GUI.
//!
//! OS-specific and Dark-theme-specific CSS code is supported via dedicated
//! mediaQueries:
//! - `prefers-color-scheme:light/dark`
//! - `os-version: macos/linux/windows`
//!
//! Rules and mediaQueries are invalidated when the file is modified by the
//! user, or when the system wide theme changes (Dark mode).
//!
//! # CSS Example
//! ```css
//! @media (prefers-color-scheme: light) {
//!   hbox {
//!     --mycolor: black;
//!   }
//! }
//!
//! @media (prefers-color-scheme: dark) {
//!   hbox {
//!     --mycolor: white;
//!   }
//! }
//!
//! hbox {
//!   color: var(--mycolor);
//!   background-color: red !important;
//! }
//!
//! scrollarea::scrollbar {
//!   width: 12px;
//! }
//!
//! @media (os-version: macos) {
//!   hbox {
//!     --toolbar-padding: 12px;
//!   }
//! }
//! ```

mod compute;
mod elements;
mod properties;
mod themes;
mod thread;
mod watchers;

pub use elements::Element;
pub use properties::ComputedProperties;
pub use thread::{parse, parse_file, parse_string, Event, Rules};

#[cfg(test)]
mod tests {
  use lightningcss::cssparser::RGBA;

  use crate::themes::{set_theme, Theme};
  use crate::{parse, ComputedProperties, Element};
  const RED: Option<RGBA> = Some(RGBA {
    red: 255,
    green: 0,
    blue: 0,
    alpha: 255,
  });
  const GREEN: Option<RGBA> = Some(RGBA {
    red: 0,
    green: 128,
    blue: 0,
    alpha: 255,
  });

  #[test]
  fn basic() {
    let source = r#"
    #foo {
      background-color: red;
    }
    hbox {
      color: green;
    }
    "#
    .to_owned();

    set_theme(Theme::Light);

    let rules = parse(source, None).unwrap();

    let elt = Element::named("hbox").id("foo");
    assert_eq!(
      rules.compute(&elt),
      ComputedProperties {
        background_color: RED,
        color: GREEN,
        ..ComputedProperties::default()
      }
    );

    let elt = Element::named("hbox");
    assert_eq!(
      rules.compute(&elt),
      ComputedProperties {
        color: GREEN,
        ..ComputedProperties::default()
      }
    );
  }

  #[test]
  fn theme() {
    let source = r#"
    @media (prefers-color-scheme: light) {
      hbox {
        color: red;
      }
    }
    @media (prefers-color-scheme: dark) {
      hbox {
        color: green;
      }
    }
    "#
    .to_owned();

    set_theme(Theme::Dark);

    let rules = parse(source, None).unwrap();

    let elt = Element::named("hbox");
    assert_eq!(
      rules.compute(&elt),
      ComputedProperties {
        color: GREEN,
        ..ComputedProperties::default()
      }
    );
  }
}
