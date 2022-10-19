mod elements;
mod thread;
mod properties;

pub use elements::Element;
pub use thread::spawn_and_parse as parse;
pub use thread::{BgParser, Event};
