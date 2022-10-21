use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crossbeam_channel::{unbounded, Receiver, Sender, select};

use anyhow::{anyhow, Result};
use lightningcss::declaration::DeclarationBlock;
use lightningcss::media_query::{MediaFeature, MediaFeatureValue, MediaQuery, Operator, Qualifier};
use lightningcss::parcel_selectors::context::QuirksMode;
use lightningcss::parcel_selectors::matching::{matches_selector, MatchingContext, MatchingMode};
use lightningcss::parcel_selectors::parser::Selector;
use lightningcss::printer::Printer;
use lightningcss::properties::custom::{CustomProperty, TokenOrValue, UnparsedProperty, Variable};
use lightningcss::properties::Property;
use lightningcss::rules::media::MediaRule;
use lightningcss::rules::CssRule;
use lightningcss::selector::Selectors;
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::values::ident::{DashedIdent, DashedIdentReference};
use log::debug;
use ouroboros::self_referencing;

use crate::infallible_send as send;

use crate::elements::Element;
use crate::properties::ComputedProperties;

use crate::watchers;

pub enum Event {
  FileChanged,
  ThemeChanged,
  /// Vec of error messages
  Parsed(ParserResult),
  Error(String),
}

impl std::fmt::Debug for Event {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Event::FileChanged => write!(f, "FileChanged"),
      Event::ThemeChanged => write!(f, "SystemColorChanged"),
      Event::Parsed(_) => write!(f, "Parsed"),
      Event::Error(_) => write!(f, "Error"),
    }
  }
}

#[self_referencing]
pub struct ParserResult {
  source: String,
  #[borrows(source)]
  #[not_covariant]
  stylesheet: StyleSheet<'this, 'this>,
}

impl<'i> ParserResult {
  pub fn compute(&self, element: &Element<'i>) -> ComputedProperties {
    self.with_stylesheet(|s| {
      crate::compute::compute(s, element)
    })
  }
}

pub fn spawn_and_parse(path: PathBuf) -> Receiver<Event> {
  let (to_main, from_css_thread) = unbounded();

  std::thread::spawn(move || {
    debug!("CSS thread spawned");

    let theme = match watchers::theme() {
      Ok(w) => w,
      Err(e) => {
        send(&to_main, Event::Error(e.to_string()));
        return;
      },
    };

    let file = match watchers::file(&path) {
      Ok(w) => w,
      Err(e) => {
        send(&to_main, Event::Error(e.to_string()));
        return;
      },
    };

    match parse(&path) {
      Ok(stylesheet) => send(&to_main, Event::Parsed(stylesheet)),
      Err(e) => send(&to_main, Event::Error(e.to_string())),
    }

    loop {
      select! {
        recv(theme.recv) -> e => {
          match e {
            Ok(watchers::ThemeEvent::Changed) => {
              send(&to_main, Event::ThemeChanged);
            },
            Err(e) => {
              send(&to_main, Event::Error(e.to_string()));
            }
          }
        },
        recv(file.recv) -> e => {
          match e {
            Ok(watchers::FileEvent::Changed) => {
              send(&to_main, Event::FileChanged);
              match parse(&path) {
                Ok(stylesheet) => send(&to_main, Event::Parsed(stylesheet)),
                Err(e) => send(&to_main, Event::Error(e.to_string())),
              }
            },
            Ok(watchers::FileEvent::Error(e)) => {
              send(&to_main, Event::Error(e));
            },
            Err(e) => {
              send(&to_main, Event::Error(e.to_string()));
            }
          }
        },
      }
    }
  });

  from_css_thread
}

fn parse(path: &Path) -> Result<ParserResult> {
  let source = std::fs::read_to_string(&path)?;
  ParserResultTryBuilder {
    source,
    stylesheet_builder: |source| {
      let options = ParserOptions {
        filename: path.to_string_lossy().to_string(),
        ..ParserOptions::default()
      };
      StyleSheet::parse(source, options).map_err(|e| anyhow!("Parsing error: {}", e))
    },
  }
  .try_build()
}
