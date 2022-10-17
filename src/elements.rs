use selectors::OpaqueElement;
use selectors::attr::*;
use selectors::matching::MatchingContext;
use selectors::matching::ElementSelectorFlags;

use std::collections::{HashMap, HashSet};

use crate::selectors::SelectorString;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PseudoClass {
  Active,
  Hover,
  Focus,
  Enabled,
  Disabled,
}

impl std::fmt::Display for PseudoClass {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, ".{}", match self {
      PseudoClass::Active => "active",
      PseudoClass::Hover => "hover",
      PseudoClass::Focus => "focus",
      PseudoClass::Enabled => "enabled",
      PseudoClass::Disabled => "disabled",
    })
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PseudoElement {
  Root,
  Scrollbar,
}

impl std::fmt::Display for PseudoElement {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "::{}", match self {
      PseudoElement::Scrollbar => "scrollbar",
      PseudoElement::Root => "root",
    })
  }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum ElementName {
  Pseudo(PseudoElement),
  Named(String),
  #[default]
  Unnamed,
}

impl std::fmt::Display for ElementName {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ElementName::Pseudo(e) => e.fmt(f),
      ElementName::Named(s) => s.fmt(f),
      ElementName::Unnamed => Ok(()),
    }
  }
}

#[derive(Debug, Default)]
pub struct Element {
  name: ElementName,
  id: Option<String>,
  classes: HashSet<String>,
  pseudo_classes: HashSet<PseudoClass>,
  attributes: HashMap<String, String>,
}

impl std::fmt::Display for Element {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self.name).and_then(|_| match &self.id {
      Some(id) => write!(f, "#{}", id),
      None => Ok(()),
    }).and_then(|_| self.classes.iter().try_for_each(|class| {
      write!(f, ".{}", class)
    })).and_then(|_| self.pseudo_classes.iter().try_for_each(|class| {
      write!(f, ":{:?}", class)
    })).and_then(|_| self.attributes.iter().try_for_each(|(name, value)| {
      write!(f, "[{}=\"{}\"]", name, value)
    }))
  }
}

impl Element {
  pub fn unamed() -> Element {
    Element::default()
  }
  pub fn root() -> Element {
    Element {
      name: ElementName::Pseudo(PseudoElement::Root),
      ..Default::default()
    }
  }
  pub fn scrollbar() -> Element {
    Element {
      name: ElementName::Pseudo(PseudoElement::Scrollbar),
      ..Default::default()
    }
  }
  pub fn named(name: impl Into<String>) -> Element {
    Element {
      name: ElementName::Named(name.into()),
      ..Default::default()
    }
  }
  pub fn id(mut self, id: impl Into<String>) -> Element {
    self.id = Some(id.into());
    self
  }
}

// FIXME: Most of these methodes are not implemented as we don't support a tree structure yet.
impl<'element> selectors::Element for &'element Element {
  type Impl = crate::selectors::CustomParser;
  fn opaque(&self) -> OpaqueElement {
    OpaqueElement::new(&self)
  }
  fn parent_element(&self) -> Option<Self> {
    None
  }
  fn parent_node_is_shadow_root(&self) -> bool {
    false
  }
  fn containing_shadow_host(&self) -> Option<Self> {
    None
  }
  fn is_pseudo_element(&self) -> bool {
    matches!(self.name, ElementName::Pseudo(_))
  }
  fn prev_sibling_element(&self) -> Option<Self> {
    None
  }
  fn next_sibling_element(&self) -> Option<Self> {
    None
  }
  fn is_html_element_in_html_document(&self) -> bool {
    false
  }
  fn has_local_name(&self, local_name: &SelectorString) -> bool {
    if let ElementName::Named(name) = &self.name {
      name == local_name.as_ref()
    } else {
      false
    }
  }

  fn has_namespace(&self, _ns: &SelectorString) -> bool {
    false
  }

  fn is_same_type(&self, other: &Self) -> bool {
    // Whether this element and the other element have the same local name and namespace.
    other.name == self.name
  }

  fn attr_matches(
    &self,
    ns: &NamespaceConstraint<&SelectorString>,
    local_name: &SelectorString,
    operation: &AttrSelectorOperation<&SelectorString>
    ) -> bool {
    // FIXME
    false
  }

  // ts == tree-structural (fist-child & such)
  fn match_non_ts_pseudo_class<F>(
    &self,
    pc: &PseudoClass,
    context: &mut MatchingContext<'_, Self::Impl>,
    flags_setter: &mut F
    ) -> bool
    where
      F: FnMut(&Self, ElementSelectorFlags) {
        self.pseudo_classes.contains(pc)
      }
  fn match_pseudo_element(
    &self,
    pe: &PseudoElement,
    context: &mut MatchingContext<'_, Self::Impl>
    ) -> bool {
    if let ElementName::Pseudo(elt) = &self.name {
      elt == pe
    } else {
      false
    }
  }
  fn is_link(&self) -> bool {
    false
  }
  fn is_html_slot_element(&self) -> bool {
    false
  }
  fn has_id(&self, id: &SelectorString, _: CaseSensitivity) -> bool {
    // Not quirks mode. Always case sensitivie
    self.id.as_ref().map(|i| i == id.as_ref()).unwrap_or(false)
  }
  fn has_class(&self, name: &SelectorString, _: CaseSensitivity) -> bool {
    // Not quirks mode. Always case sensitivie
    self.classes.contains(name.as_ref())
  }
  fn imported_part( &self, name: &SelectorString) -> Option<SelectorString> {
    None
  }
  fn is_part(&self, name: &SelectorString) -> bool {
    false
  }
  fn is_empty(&self) -> bool {
    true
  }
  fn is_root(&self) -> bool {
    if let ElementName::Pseudo(elt) = &self.name {
      *elt == PseudoElement::Root
    } else {
      false
    }
  }
}
