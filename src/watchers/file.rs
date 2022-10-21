use std::path::Path;
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::error;

use anyhow::Result;

use notify::event::{DataChange, EventKind, ModifyKind};
use notify::{RecursiveMode, FsEventWatcher};
use notify::Watcher as NotifyWatcher;

pub struct Watcher {
  _inner: FsEventWatcher,
  pub recv: Receiver<Event>,
}

#[derive(Debug)]
pub enum Event {
  FileChanged,
  Error(String),
}

fn send<T>(sender: &Sender<T>, event: T) {
  if let Err(e) = sender.send(event) {
    error!("Sending message to css thread failed: {}", e);
  }
}

pub fn watch(path: &Path) -> Result<Watcher> {
  let (to_parent_thread, from_this_thread) = unbounded();
  let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
    match res {
      Ok(e) => {
        if matches!(e.kind, EventKind::Modify(ModifyKind::Data(DataChange::Content))) {
          send(&to_parent_thread, Event::FileChanged);
        }
      },
      Err(e) => {
        send(&to_parent_thread, Event::Error(e.to_string()));
      },
    }
  })?;

  watcher.watch(&path, RecursiveMode::NonRecursive)?;
  Ok(Watcher {
    _inner: watcher,
    recv: from_this_thread,
  })
}
