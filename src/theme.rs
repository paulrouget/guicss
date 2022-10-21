use objc2::foundation::{NSArray, NSString};

#[derive(Clone, Copy, Debug)]
pub enum Theme {
  Dark,
  Light,
}

pub fn get_theme() -> Theme {
  // FIXME: ensure this runs in main thread
  let app = crate::nsapp::NSApp();
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
