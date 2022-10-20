use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

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
use notify::event::{DataChange, EventKind, ModifyKind};
use notify::{RecursiveMode, Watcher};
use ouroboros::self_referencing;

use crate::elements::Element;
use crate::properties::ComputedProperties;

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
    let mut variables = HashMap::new();
    let mut computed = ComputedProperties::default();

    self.with_stylesheet(|s| {
      let mut rules: Vec<(&Selector<Selectors>, &DeclarationBlock)> = s
        .rules
        .0
        .iter()
        .filter_map(|rule| {
          match rule {
            CssRule::Style(style) => Some([style].to_vec()),
            CssRule::Media(MediaRule { query, rules, .. }) => {
              let matches = query.media_queries.iter().any(
                |MediaQuery {
                   qualifier,
                   media_type: _,
                   condition,
                 }| {
                  match qualifier {
                    Some(Qualifier::Not) => !condition.as_ref().map(|c| check_media_query(c)).unwrap_or(true),
                    _ => condition.as_ref().map(|c| check_media_query(c)).unwrap_or(true),
                  }
                },
              );
              if matches {
                // FIXME: only keeping on nesting level of media queries
                Some(
                  rules
                    .0
                    .iter()
                    .filter_map(|r| {
                      match r {
                        CssRule::Style(style) => Some(style),
                        _ => None,
                      }
                    })
                    .collect(),
                )
              } else {
                None
              }
            },
            unknown => {
              println!("Unsupported: {:?}", unknown);
              None
            },
          }
        })
        .flatten()
        .map(|style| style.selectors.0.iter().map(|s| (s, &style.declarations)))
        .flatten()
        .collect();
      rules.sort_by(|(s1, _), (s2, _)| s1.specificity().cmp(&s2.specificity()));

      let mut context = MatchingContext::new(MatchingMode::Normal, None, None, QuirksMode::NoQuirks);
      let (normal, important): (Vec<_>, Vec<_>) = rules
        .into_iter()
        .filter_map(|(s, decs)| {
          if matches_selector(s, 0, None, &element, &mut context, &mut |_, _| {}) {
            Some((&decs.declarations, &decs.important_declarations))
          } else {
            None
          }
        })
        .unzip();

      let normal = normal.into_iter().flatten();
      let important = important.into_iter().flatten();

      let all = normal.chain(important);

      let without_var: Vec<_> = all
        .filter(|prop| {
          if let Property::Custom(CustomProperty { name, value: tokens }) = prop {
            if name.starts_with("--") {
              let mut source = String::new();
              let mut printer = Printer::new(&mut source, PrinterOptions::default());
              tokens.to_css(&mut printer, false).unwrap();
              variables.insert(name.clone(), source.clone());
              return false;
            }
          }
          true
        })
        .collect();

      for prop in without_var {
        if let Property::Unparsed(UnparsedProperty {
          property_id: id,
          value: tokens,
        }) = prop
        {
          if let Some(token) = tokens.0.get(0) {
            if let TokenOrValue::Var(Variable {
              name: DashedIdentReference { ident: DashedIdent(name), .. },
              ..
            }) = token
            {
              if let Some(source) = variables.get(name) {
                if let Ok(prop) = Property::parse_string(id.clone(), &source, ParserOptions::default()) {
                  if let Err(e) = computed.apply(&prop) {
                    eprintln!("{}", e);
                  }
                  continue;
                } else {
                  eprintln!("Could not parse `{}` variable content ({}) for property {:?}", name, source, prop);
                }
              } else {
                eprintln!("Could not resolve variable: {}", name);
              }
            }
          }
        }
        if let Err(e) = computed.apply(&prop) {
          eprintln!("{}", e);
        }
      }
      println!("Computed property: {:?}", computed);
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

fn check_media_query(condition: &lightningcss::media_query::MediaCondition<'_>) -> bool {
  use lightningcss::media_query::MediaCondition::*;
  match condition {
    Feature(MediaFeature::Plain {
      name,
      value: MediaFeatureValue::Ident(ident),
    }) => {
      match name.as_ref() {
        "os-version" => ident.as_ref() == std::env::consts::OS,
        "prefers-color-scheme" => ident.as_ref() == "light", // FIXME
        _ => false,
      }
    },
    Not(cond) => !check_media_query(&cond),
    Operation(conditions, Operator::And) => conditions.iter().all(check_media_query),
    Operation(conditions, Operator::Or) => conditions.iter().any(check_media_query),
    InParens(condition) => check_media_query(condition),
    _ => {
      // Unsupported
      false
    },
  }
}
