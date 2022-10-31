use crate::element::Element;

/// Id and classes matched against selectors.
/// Supports up to 16 classes.
#[derive(Clone, Copy, Debug, Default)]
pub struct IdAndClasses {
  pub id: Option<&'static str>,
  // Copy trait (required by Iced) make it impossible
  // to use Vec<>
  pub classes: [Option<&'static str>; 16],
}

impl IdAndClasses {
  pub fn parse(s: &'static str) -> IdAndClasses {
    use regex::Regex;
    lazy_static::lazy_static! {
      static ref ID: Regex = Regex::new(
        r"\#([a-zA-Z][0-9a-zA-Z_]*)"
        ).unwrap();
      static ref CLASSES: Regex = Regex::new(
        r"\.([a-zA-Z][0-9a-zA-Z_]*)"
        ).unwrap();
    }

    let mut def = IdAndClasses {
      id: ID.captures(s).and_then(|c| c.get(1)).map(|s| s.as_str()),
      ..IdAndClasses::default()
    };

    CLASSES.captures_iter(s).for_each(|capture| {
      if let Some(class) = capture.get(1) {
        def.add_class(class.as_str());
      }
    });

    def
  }

  pub fn set_id(mut self, id: &'static str) {
    self.id = Some(id);
  }

  pub fn add_class(&mut self, class: &'static str) {
    let free = self.classes.iter_mut().find(|i| i.is_none());
    if let Some(free) = free {
      *free = Some(class);
    }
  }
}

impl<'i> Element<'i> {
  pub(crate) fn def(name: &'i str, def: &IdAndClasses) -> Element<'i> {
    let mut elt = Element::named(name);
    elt.id = def.id;
    def.classes.into_iter().flatten().for_each(|c| {
      elt.classes.insert(c);
    });
    elt
  }
}
