mod compute;
mod elements;
mod nsapp;
mod properties;
mod thread;
mod watchers;

use crossbeam_channel::Sender;
pub use elements::Element;
use log::error;
pub use thread::{spawn_and_parse as parse, Event};
pub use watchers::theme::get_theme;

/// For when we can't report errors.
pub(crate) fn infallible_send<T>(sender: &Sender<T>, event: T) {
  if let Err(e) = sender.send(event) {
    error!("Sending message to css thread failed: {}", e);
  }
}
