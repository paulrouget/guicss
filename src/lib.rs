//! `GuiCss` is a CSS parser designed for Rust Desktop GUI.
//!
//! **Warning:** Work In Progress.
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
//! ```no_run
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
  const RED_COLOR: Option<RGBA> = Some(RGBA {
    red: 255,
    green: 0,
    blue: 0,
    alpha: 255,
  });
  const GREEN_COLOR: Option<RGBA> = Some(RGBA {
    red: 0,
    green: 128,
    blue: 0,
    alpha: 255,
  });

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

    set_theme(Theme::Light);

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

    set_theme(Theme::Dark);

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
