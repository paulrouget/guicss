#![allow(non_snake_case)]

use objc2::foundation::{MainThreadMarker, NSArray, NSObject, NSString};
use objc2::rc::{Id, Shared};
use objc2::{extern_class, extern_methods, msg_send_id, ClassType};

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

type NSAppearanceName = NSString;

extern_methods!(
  unsafe impl NSAppearance {
    pub fn bestMatchFromAppearancesWithNames(&self, appearances: &NSArray<NSAppearanceName>) -> Id<NSAppearanceName, Shared> {
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
  // TODO: Only allow access from main thread
  NSApplication::shared(unsafe { MainThreadMarker::new_unchecked() })
}

extern_methods!(
  unsafe impl NSApplication {
    /// This can only be called on the main thread since it may initialize
    /// the application and since it's parameters may be changed by the main
    /// thread at any time (hence it is only safe to access on the main thread).
    pub fn shared(_mtm: MainThreadMarker) -> Id<Self, Shared> {
      let app: Option<_> = unsafe { msg_send_id![Self::class(), sharedApplication] };
      // SAFETY: `sharedApplication` always initializes the app if it isn't already
      unsafe { app.unwrap_unchecked() }
    }

    pub fn effectiveAppearance(&self) -> Id<NSAppearance, Shared> {
      unsafe { msg_send_id![self, effectiveAppearance] }
    }
  }
);
