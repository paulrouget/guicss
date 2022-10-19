use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use ouroboros::self_referencing;

use anyhow::Result;

use log::debug;
use notify::event::{DataChange, EventKind, ModifyKind};
use notify::{RecursiveMode, Watcher};

use lightningcss::rules::CssRule;
use lightningcss::stylesheet::{StyleSheet, ParserOptions};
use lightningcss::declaration::DeclarationBlock;
use lightningcss::parcel_selectors::context::QuirksMode;
use lightningcss::parcel_selectors::matching::{MatchingContext, MatchingMode, matches_selector};
use lightningcss::parcel_selectors::parser::Selector;
use lightningcss::selector::Selectors;

use crate::elements::Element;

pub enum Event {
  FileChanged,
  SystemColorChanged,
  /// Vec of error messages
  Parsed(ParserResult),
  Error(String),
}

impl std::fmt::Debug for Event {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Event::FileChanged => write!(f, "FileChanged"),
      Event::SystemColorChanged => write!(f, "SystemColorChanged"),
      Event::Parsed(_) => write!(f, "Parsed"),
      Event::Error(_) => write!(f, "Error"),
    }
  }
}

enum WatcherEvent {
  FileChanged,
  Error(String),
}

pub struct BgParser {
  pub thread: Receiver<Event>,
}

#[self_referencing]
pub struct ParserResult {
  source: String,
  #[borrows(source)]
  #[not_covariant]
  stylesheet: StyleSheet<'this, 'this>,
}

impl<'i> ParserResult {
  pub fn compute(&self, element: &Element<'i>) {
    self.with_stylesheet(|s| {
      let mut rules: Vec<(&Selector<Selectors>, &DeclarationBlock)> = s.rules.0.iter().filter_map(|rule| {
        match rule {
          CssRule::Style(style) => Some(style),
          _ => None,
        }
      }).map(|style| {
        style.selectors.0.iter().map(|s| {
          (s, &style.declarations)
        })
      }).flatten().collect();
      rules.sort_by(|(s1, _), (s2, _)| {
        s1.specificity().cmp(&s2.specificity())
      });
      rules.iter().filter(|(s, _)| {
        let mut context = MatchingContext::new(MatchingMode::Normal, None, None, QuirksMode::NoQuirks);
        matches_selector(s, 0, None, &element, &mut context, &mut |_, _| {})
      }).for_each(|x| todo!());

    })
  }
}

fn send<T>(sender: &Sender<T>, event: T) {
  if let Err(e) = sender.send(event) {
    eprintln!("Sending message to thread failed: {}", e);
  }
}

pub fn spawn_and_parse(path: PathBuf) -> BgParser {
  let (to_main, from_css_thread) = channel();

  std::thread::spawn(move || {
    debug!("CSS thread spawned");

    let (to_css_thread, from_watcher_thread) = channel();

    let watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
      match res {
        Ok(e) => {
          if matches!(e.kind, EventKind::Modify(ModifyKind::Data(DataChange::Content))) {
            send(&to_css_thread, WatcherEvent::FileChanged);
          }
        },
        Err(e) => {
          send(&to_css_thread, WatcherEvent::Error(e.to_string()));
        },
      }
    });

    let mut watcher = match watcher {
      Ok(w) => w,
      Err(e) => {
        send(&to_main, Event::Error(e.to_string()));
        return;
      },
    };

    if let Err(e) = watcher.watch(&path, RecursiveMode::NonRecursive) {
      send(&to_main, Event::Error(e.to_string()));
    }

    match parse(&path) {
      Ok(stylesheet) => send(&to_main, Event::Parsed(stylesheet)),
      Err(e) => send(&to_main, Event::Error(e.to_string())),
    }

    loop {
      let event = from_watcher_thread.recv();
      match event {
        Ok(WatcherEvent::FileChanged) => {
          send(&to_main, Event::FileChanged);
          match parse(&path) {
            Ok(stylesheet) => send(&to_main, Event::Parsed(stylesheet)),
            Err(e) => send(&to_main, Event::Error(e.to_string())),
          }
        },
        Ok(WatcherEvent::Error(e)) => {
          send(&to_main, Event::Error(e));
        },
        Err(e) => {
          send(&to_main, Event::Error(e.to_string()));
        },
      }
    }
  });

  BgParser { thread: from_css_thread }
}

fn parse(path: &Path) -> Result<ParserResult> {
  let source = std::fs::read_to_string(&path)?;
  let parser_result = ParserResultBuilder {
    source,
    stylesheet_builder: |source| {
      let options = ParserOptions {
        filename: path.to_string_lossy().to_string(),
        ..ParserOptions::default()
      };
      StyleSheet::parse(source, options).unwrap()
    },
  }.build();

  Ok(parser_result)
}
