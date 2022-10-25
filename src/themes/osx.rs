#![allow(non_snake_case)]

use std::ptr;

use anyhow::{bail, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::error;
use objc2::foundation::{is_main_thread, MainThreadMarker, NSArray, NSObject, NSString};
use objc2::rc::{Id, Shared};
use objc2::runtime::Object;
use objc2::{class, declare_class, extern_class, extern_methods, msg_send, msg_send_id, sel, ClassType};

use crate::themes::{Event, SystemTheme};

// FIXME: Event sender is unsafe #2
static mut SENDER: Option<Sender<Event>> = None;

pub(crate) fn get_system_theme() -> SystemTheme {
  let get_theme = || {
    let app = NSApp();
    let appearance = app.effectiveAppearance();
    let aqua = NSString::from_str("NSAppearanceNameAqua");
    let dark_aqua = NSString::from_str("NSAppearanceNameDarkAqua");
    let names = &NSArray::from_slice(&[aqua, dark_aqua]);
    let name = appearance.bestMatchFromAppearancesWithNames(names);
    match &*name.to_string() {
      "NSAppearanceNameDarkAqua" => SystemTheme::Dark,
      _ => SystemTheme::Light,
    }
  };

  if is_main_thread() {
    get_theme()
  } else {
    dispatch::Queue::main().exec_sync(get_theme)
  }
}

extern_class!(
  #[derive(Debug, PartialEq, Eq, Hash)]
  pub(crate) struct NSResponder;

  unsafe impl ClassType for NSResponder {
    type Super = NSObject;
  }
);

extern_class!(
  #[derive(Debug, PartialEq, Eq, Hash)]
  pub(crate) struct NSAppearance;

  unsafe impl ClassType for NSAppearance {
    type Super = NSObject;
  }
);

extern_methods!(
  unsafe impl NSAppearance {
    pub(crate) fn bestMatchFromAppearancesWithNames(&self, appearances: &NSArray<NSString>) -> Id<NSString, Shared> {
      unsafe { msg_send_id![self, bestMatchFromAppearancesWithNames: appearances,] }
    }
  }
);

extern_class!(
  #[derive(Debug, PartialEq, Eq, Hash)]
  pub(crate) struct NSApplication;

  unsafe impl ClassType for NSApplication {
    #[inherits(NSObject)]
    type Super = NSResponder;
  }
);

pub(crate) fn NSApp() -> Id<NSApplication, Shared> {
  NSApplication::shared(unsafe { MainThreadMarker::new_unchecked() })
}

extern_methods!(
  unsafe impl NSApplication {
    pub(crate) fn shared(_mtm: MainThreadMarker) -> Id<Self, Shared> {
      let app: Option<_> = unsafe { msg_send_id![Self::class(), sharedApplication] };
      unsafe { app.unwrap_unchecked() }
    }

    pub(crate) fn effectiveAppearance(&self) -> Id<NSAppearance, Shared> {
      unsafe { msg_send_id![self, effectiveAppearance] }
    }
  }
);

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
        if let Err(e) = s.send(Event::Changed) {
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

/// Sends `Event::Changed` when system-wide theme changed.
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
