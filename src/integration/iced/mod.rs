mod converters;
mod css;
mod id_and_classes;
mod shared_rules;

pub use css::CSS;
pub use id_and_classes::IdAndClasses;
pub use shared_rules::SharedRules;

#[derive(Clone, Debug)]
pub enum CssEvent {
  Error(String),
  Invalidated,
}
