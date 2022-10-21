mod elements;
mod properties;
mod thread;
mod watchers;
mod compute;
mod nsapp;

pub use elements::Element;
pub use thread::{spawn_and_parse as parse, Event};
pub use watchers::theme::get_theme;

use crossbeam_channel::Sender;
use log::error;

/// For when we can't report errors.
pub(crate) fn infallible_send<T>(sender: &Sender<T>, event: T) {
  if let Err(e) = sender.send(event) {
    error!("Sending message to css thread failed: {}", e);
  }
}

