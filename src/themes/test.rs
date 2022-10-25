/// Basic non-threaded theme getter / watcher for testing purpose.
use std::cell::{Cell, RefCell};

use anyhow::{bail, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::themes::{Event, SystemTheme};

thread_local! {
  static THEME: Cell<SystemTheme> = Cell::new(SystemTheme::Light);
  static SENDER: RefCell<Option<Sender<Event>>> = RefCell::new(None);
}

#[cfg(test)]
pub(crate) fn set_theme(theme: SystemTheme) {
  THEME.with(|t| t.set(theme));
  SENDER.with(|t| {
    if let Some(sender) = t.borrow().as_ref() {
      sender.send(Event::Changed).unwrap();
    }
  });
}

pub(crate) fn get_system_theme() -> SystemTheme {
  THEME.with(|t| t.get())
}

pub(crate) struct Watcher {
  pub(crate) recv: Receiver<Event>,
}

/// Sends `Event::Invalidated` when system-wide theme changed.
///
/// # Errors
///
/// Will fail if a watcher was already started.
pub(crate) fn watch() -> Result<Watcher> {
  let (to, recv) = unbounded();

  SENDER
    .with(|s| {
      if s.borrow().is_some() {
        bail!("Watcher already initialized");
      }
      *s.borrow_mut() = Some(to);
      Ok(())
    })
    .map(|_| Watcher { recv })
}
