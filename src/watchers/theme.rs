use std::ptr;

use anyhow::{bail, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::error;
use objc2::foundation::{NSObject, NSString};
use objc2::rc::{Id, Shared};
use objc2::runtime::Object;
use objc2::{class, declare_class, msg_send, msg_send_id, sel, ClassType};

// FIXME: lame. Should be attached to the Delegate.
static mut SENDER: Option<Sender<Event>> = None;

declare_class!(
  #[derive(Debug)]
  pub(crate) struct Delegate {}

  unsafe impl ClassType for Delegate {
    type Super = NSObject;
  }

  unsafe impl Delegate {
    #[sel(init_watcher)]
    fn init_watcher(&mut self) -> Option<&mut Self> {
      let this: Option<&mut Self> = unsafe { msg_send![self, init] };
      this.map(|this| {
        let notification_center: Id<Object, Shared> = unsafe { msg_send_id![class!(NSDistributedNotificationCenter), defaultCenter] };
        let notification_name = NSString::from_str("AppleInterfaceThemeChangedNotification");
        unsafe {
          let _: () = msg_send![
            &notification_center,
            addObserver: &*this
              selector: sel!(effectiveAppearanceDidChange:)
              name: &*notification_name
              object: ptr::null::<Object>()
          ];
        }
        this
      })
    }

    #[sel(effectiveAppearanceDidChange:)]
    fn effective_appearance_did_change(&self, _sender: Option<&Object>) {
      if let Some(s) = unsafe { &SENDER } {
        if let Err(e) = s.send(Event::Invalidated) {
          error!("Sending message to css thread failed: {}", e);
        }
      }
    }
  }
);

pub(crate) struct Watcher {
  _inner: Id<Delegate, Shared>,
  pub(crate) recv: Receiver<Event>,
}

#[derive(Debug)]
pub(crate) enum Event {
  Invalidated,
}

/// Sends `Event::Invalidated` when system-wide theme changed.
///
/// # Errors
///
/// Will fail if a watcher was already started.
pub(crate) fn watch() -> Result<Watcher> {
  let (to_parent_thread, from_this_thread) = unbounded();

  unsafe {
    if SENDER.is_some() {
      bail!("Watcher already initialized");
    }
    SENDER = Some(to_parent_thread);
  }

  let watcher = unsafe { msg_send_id![msg_send_id![Delegate::class(), alloc], init_watcher] };

  Ok(Watcher {
    _inner: watcher,
    recv: from_this_thread,
  })
}
