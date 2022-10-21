use std::path::Path;

use anyhow::Result;
use log::error;
use crossbeam_channel::{unbounded, Receiver};
use notify::event::{DataChange, EventKind, ModifyKind};
use notify::{FsEventWatcher, RecursiveMode, Watcher as NotifyWatcher};

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
    let sent_op = match res {
      Ok(e) => {
        if matches!(e.kind, EventKind::Modify(ModifyKind::Data(DataChange::Content))) {
          to_parent_thread.send(Event::Changed)
        } else {
          Ok(())
        }
      },
      Err(e) => {
        to_parent_thread.send(Event::Error(e.to_string()))
      },
    };
    if let Err(e) = sent_op {
      error!("Sending message failed: {}", e);
    }
  })?;

  watcher.watch(path, RecursiveMode::NonRecursive)?;
  Ok(Watcher {
    _inner: watcher,
    recv: from_this_thread,
  })
}
