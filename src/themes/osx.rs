#![allow(non_snake_case)]

use objc2::foundation::{MainThreadMarker, NSArray, NSObject, NSString};
use objc2::rc::{Id, Shared};
use objc2::{extern_class, extern_methods, msg_send_id, ClassType};

use crate::themes::Theme;

pub(crate) fn get_theme() -> Theme {
  // FIXME: ensure this runs in main thread
  let app = NSApp();
  let appearance = app.effectiveAppearance();
  let name = appearance.bestMatchFromAppearancesWithNames(&NSArray::from_slice(&[
    NSString::from_str("NSAppearanceNameAqua"),
    NSString::from_str("NSAppearanceNameDarkAqua"),
  ]));
  match &*name.to_string() {
    "NSAppearanceNameDarkAqua" => Theme::Dark,
    _ => Theme::Light,
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
  // FIXME: Only allow access from main thread
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
