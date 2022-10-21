use std::path::Path;

use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver};
use notify::event::{DataChange, EventKind, ModifyKind};
use notify::{FsEventWatcher, RecursiveMode, Watcher as NotifyWatcher};

use crate::infallible_send as send;

pub struct Watcher {
  _inner: FsEventWatcher,
  pub recv: Receiver<Event>,
}

#[derive(Debug)]
pub enum Event {
  Changed,
  Error(String),
}

pub fn watch(path: &Path) -> Result<Watcher> {
  let (to_parent_thread, from_this_thread) = unbounded();
  let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
    match res {
      Ok(e) => {
        if matches!(e.kind, EventKind::Modify(ModifyKind::Data(DataChange::Content))) {
          send(&to_parent_thread, Event::Changed);
        }
      },
      Err(e) => {
        send(&to_parent_thread, Event::Error(e.to_string()));
      },
    }
  })?;

  watcher.watch(path, RecursiveMode::NonRecursive)?;
  Ok(Watcher {
    _inner: watcher,
    recv: from_this_thread,
  })
}
