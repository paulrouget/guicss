//! `GuiCss` is a CSS parser designed for Rust Desktop GUI.
//!
//! **Warning:** Work In Progress.
//!
//! The idea is to make it easier to theme any Rust GUI, iterate faster,
//! or offer theme customisation to the end user.
//!
//! # Features
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
//! ```no_run
#![doc = include_str!("../examples/winit.rs")]
//! ```

mod compute;
/// Elements matched against selectors.
pub mod element;
mod file_watcher;
/// Helpers for toolkits.
pub mod integration;
/// Parsing operations.
pub mod parser;
/// Parsed and computed properties.
pub mod properties;
mod themes;

#[cfg(feature = "toolkit-iced")]
pub use iced;
#[cfg(feature = "toolkit-iced")]
pub use iced_native;

#[cfg(test)]
mod tests {
  use crate::element::Element;
  use crate::parser::parse_string_sync as parse;
  use crate::properties::{Color, ComputedProperties};
  use crate::themes::{set_theme, SystemTheme};
  const RED_COLOR: Color = Color { r: 255, g: 0, b: 0, a: 255 };
  const GREEN_COLOR: Color = Color { r: 0, g: 128, b: 0, a: 255 };

  fn green_prop() -> ComputedProperties {
    ComputedProperties {
      color: GREEN_COLOR,
      ..ComputedProperties::default()
    }
  }

  fn red_prop() -> ComputedProperties {
    ComputedProperties {
      color: RED_COLOR,
      ..ComputedProperties::default()
    }
  }

  #[test]
  fn basic() {
    let source = r#"
    #foo {
      background-color: red;
    }
    hbox {
      color: green;
    }
    "#;

    set_theme(SystemTheme::Light);

    let rules = parse(source, None).unwrap();

    let elt = Element::named("hbox").id("foo");
    assert_eq!(
      rules.compute(&elt),
      ComputedProperties {
        background_color: RED_COLOR,
        color: GREEN_COLOR,
        ..ComputedProperties::default()
      }
    );

    let elt = Element::named("hbox");
    assert_eq!(rules.compute(&elt), green_prop());
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
    "#;

    set_theme(SystemTheme::Dark);

    let rules = parse(source, None).unwrap();

    let elt = Element::named("hbox");
    assert_eq!(rules.compute(&elt), green_prop());
  }

  #[test]
  fn attributes_and_classes() {
    let r1 = parse("hbox[foo=bar] { color: red; }", None).unwrap();
    let r2 = parse("hbox.foo.bar { color: green; }", None).unwrap();
    let elt1 = Element::named("hbox").attribute("foo", "bar");
    let elt2 = Element::named("hbox").class("foo").class("bar");
    assert_eq!(r1.compute(&elt1), red_prop());
    assert_eq!(r2.compute(&elt2), green_prop());
  }
}
