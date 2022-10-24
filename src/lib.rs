//! `GuiCss` is a CSS parser designed for Rust Desktop GUI.
//!
//! The idea is to make it easier to theme any Rust GUI, iterate faster,
//! or offer theme customisation to the end user.
//!
//! Features:
//! - The parser recompiles the CSS file as the user modifies CSS file;
//! - Parsing runs in its dedicated thread;
//! - The parser supports mediaQueries to write platform specific code
//!   (`os-version: macos|linux|windows`) and to match the OS theme
//!   (`prefers-color-scheme: light|dark`);
//! - Computed properties are exported to a generic format that can be use with
//!   any toolkit. It also supports exporting to toolkit-specific style
//!   structures;
//! - CSS variables are supported;
//!
//! # CSS example
//! ```css
#![doc = include_str!("../examples/basic.css")]
//! ```
//! # Example with [winit](https://lib.rs/winit)
//! ```rust
#![doc = include_str!("../examples/basic.rs")]
//! ```

mod compute;
mod elements;
mod file_watcher;
mod properties;
mod themes;
mod thread;

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
