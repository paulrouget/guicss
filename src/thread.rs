use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use crossbeam_channel::select;
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use log::debug;
use ouroboros::self_referencing;

use crate::elements::Element;
use crate::properties::ComputedProperties;
use crate::watchers;

pub enum Event {
  FileChanged(ParserResult),
  ThemeChanged,
  Error(String),
}

impl std::fmt::Debug for Event {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Event::FileChanged(_) => write!(f, "FileChanged"),
      Event::ThemeChanged => write!(f, "SystemColorChanged"),
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
  pub fn compute(&self, element: &Element<'i>, theme: crate::theme::Theme) -> ComputedProperties {
    self.with_stylesheet(|s| crate::compute::compute(s, element, theme))
  }
}

pub fn spawn_and_parse<F>(path: PathBuf, cb: F)
where F: Fn(Event) + Send + 'static {
  std::thread::spawn(move || {
    debug!("CSS thread spawned");

    let theme = match watchers::theme::watch() {
      Ok(w) => w,
      Err(e) => {
        cb(Event::Error(e.to_string()));
        return;
      },
    };

    let file = match watchers::file::watch(&path) {
      Ok(w) => w,
      Err(e) => {
        cb(Event::Error(e.to_string()));
        return;
      },
    };

    match parse(&path) {
      Ok(stylesheet) => cb(Event::FileChanged(stylesheet)),
      Err(e) => cb(Event::Error(e.to_string())),
    }

    loop {
      select! {
        recv(theme.recv) -> e => {
          match e {
            Ok(watchers::theme::Event::Changed) => {
              cb(Event::ThemeChanged);
            },
            Err(e) => {
              cb(Event::Error(e.to_string()));
            }
          }
        },
        recv(file.recv) -> e => {
          match e {
            Ok(watchers::file::Event::Changed) => {
              match parse(&path) {
                Ok(stylesheet) => cb(Event::FileChanged(stylesheet)),
                Err(e) => cb(Event::Error(e.to_string())),
              }
            },
            Ok(watchers::file::Event::Error(e)) => {
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
