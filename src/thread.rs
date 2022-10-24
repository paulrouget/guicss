use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use crossbeam_channel::select;
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use log::debug;
use ouroboros::self_referencing;

use crate::compute::{pre_compute, PreComputedRules};
use crate::elements::Element;
use crate::file_watcher::{watch as watch_file, Event as file_event};
use crate::properties::ComputedProperties;
use crate::themes::{watch as watch_theme, Event as theme_event};

/// Events sent from CSS thread.
pub enum Event {
  /// File has changed, or mediaQueries have been
  /// invalidated. Restyling is necessary.
  Invalidated(Rules),
  /// An error occured during the parsing or watching process.
  Error(String),
}

impl std::fmt::Debug for Event {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Event::Invalidated(_) => write!(f, "Invalidated(..)"),
      Event::Error(_) => write!(f, "Error(..)"),
    }
  }
}

/// CSS parse result.
pub struct Rules(ParserResult);

impl Rules {
  /// Compute properties of element.
  pub fn compute(&self, element: &Element<'_>) -> ComputedProperties {
    self.0.with_rules(|s| s.compute(element))
  }
}

#[self_referencing]
struct ParserResult {
  source: String,
  #[borrows(source)]
  #[not_covariant]
  rules: PreComputedRules<'this>,
}

fn read_and_parse(path: &Path) -> Result<Rules> {
  let source = read_to_string(path)?;
  parse(source, Some(path))
}

/// Parse string.
/// # Errors
/// Parsing errors.
pub fn parse(source: String, path: Option<&Path>) -> Result<Rules> {
  ParserResultTryBuilder {
    source,
    rules_builder: |source| {
      let options = match path {
        Some(path) => {
          ParserOptions {
            filename: path.to_string_lossy().to_string(),
            ..ParserOptions::default()
          }
        },
        None => ParserOptions::default(),
      };
      let stylesheet = StyleSheet::parse(source, options).map_err(|e| anyhow!("Parsing error: {}", e))?;
      Ok(pre_compute(stylesheet))
    },
  }
  .try_build()
  .map(Rules)
}

/// Parse string. Event are sent via the closure.
/// Closure is run in different thread.
pub fn parse_string<F>(source: String, cb: F)
where F: Fn(Event) + Send + 'static {
  std::thread::spawn(move || {
    debug!("CSS thread spawned");

    let theme = match watch_theme() {
      Ok(w) => w,
      Err(e) => {
        cb(Event::Error(e.to_string()));
        return;
      },
    };

    match parse(source.clone(), None) {
      Ok(rules) => cb(Event::Invalidated(rules)),
      Err(e) => cb(Event::Error(e.to_string())),
    }

    loop {
      match theme.recv.recv() {
        Ok(theme_event::Invalidated) => {
          // FIXME: re-parsing is not necessary.
          // Only pre_compute should be called here.
          match parse(source.clone(), None) {
            Ok(rules) => cb(Event::Invalidated(rules)),
            Err(e) => cb(Event::Error(e.to_string())),
          }
        },
        Err(e) => {
          cb(Event::Error(e.to_string()));
        },
      }
    }
  });
}

/// Parse and watch a file. Event are sent via the closure.
/// Closure is run in different thread.
pub fn parse_file<F>(path: PathBuf, cb: F)
where F: Fn(Event) + Send + 'static {
  std::thread::spawn(move || {
    debug!("CSS thread spawned");

    let theme = match watch_theme() {
      Ok(w) => w,
      Err(e) => {
        cb(Event::Error(e.to_string()));
        return;
      },
    };

    let file = match watch_file(&path) {
      Ok(w) => w,
      Err(e) => {
        cb(Event::Error(e.to_string()));
        return;
      },
    };

    match read_and_parse(&path) {
      Ok(rules) => cb(Event::Invalidated(rules)),
      Err(e) => cb(Event::Error(e.to_string())),
    }

    loop {
      select! {
        recv(theme.recv) -> e => {
          match e {
            Ok(theme_event::Invalidated) => {
              // FIXME: re-parsing is not necessary.
              // Only pre_compute should be called here.
              match read_and_parse(&path) {
                Ok(rules) => cb(Event::Invalidated(rules)),
                Err(e) => cb(Event::Error(e.to_string())),
              }
            },
            Err(e) => {
              cb(Event::Error(e.to_string()));
            }
          }
        },
        recv(file.recv) -> e => {
          match e {
            Ok(file_event::Invalidated) => {
              match read_and_parse(&path) {
                Ok(rules) => cb(Event::Invalidated(rules)),
                Err(e) => cb(Event::Error(e.to_string())),
              }
            },
            Ok(file_event::Error(e)) => {
              cb(Event::Error(e));
            },
            Err(e) => {
              cb(Event::Error(e.to_string()));
            }
          }
        },
      }
    }
  });
}
