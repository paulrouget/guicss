use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

use log::debug;
use notify::event::{DataChange, EventKind, ModifyKind};
use notify::{RecursiveMode, Watcher};

use crate::elements::Element;
use crate::parser::Rule;
use crate::properties::ComputedProperties;

#[derive(Debug)]
pub enum Event {
  FileChanged,
  SystemColorChanged,
  /// Vec of error messages
  Parsed,
  WatchError(String),
  ThreadError(String),
  FSError(String),
}

enum WatcherEvent {
  FileChanged,
  Error(String),
}

pub struct StyleSheet {
  pub thread: Receiver<Event>,
}

impl StyleSheet {
  pub fn compute(&self, elt: &Element) -> ComputedProperties {
    ComputedProperties {
      padding_top: 0.0,
    }
  }
}

fn send<T>(sender: &Sender<T>, event: T) {
  if let Err(e) = sender.send(event) {
    eprintln!("Sending message to thread failed: {}", e);
  }
}

pub fn parse(path: PathBuf) -> StyleSheet {
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
        send(&to_main, Event::WatchError(e.to_string()));
        return;
      },
    };

    if let Err(e) = watcher.watch(&path, RecursiveMode::NonRecursive) {
      send(&to_main, Event::WatchError(e.to_string()));
    }

    let source = std::fs::read_to_string(&path);

    match source {
      Err(e) => {
        send(&to_main, Event::FSError(e.to_string()));
      },
      Ok(source) => {
        let rules = crate::parser::parse(&source).rules;

        send(&to_main, Event::Parsed);
      },
    }

    loop {
      let event = from_watcher_thread.recv();
      match event {
        Ok(WatcherEvent::FileChanged) => {
          send(&to_main, Event::FileChanged);
        },
        Ok(WatcherEvent::Error(e)) => {
          send(&to_main, Event::WatchError(e));
        },
        Err(e) => {
          send(&to_main, Event::ThreadError(e.to_string()));
        },
      }
    }
  });

  StyleSheet { thread: from_css_thread }
}
