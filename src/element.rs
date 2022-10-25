use std::collections::{HashMap, HashSet};

use lightningcss::cssparser::ToCss;
use lightningcss::parcel_selectors;
use lightningcss::parcel_selectors::attr::{AttrSelectorOperation, AttrSelectorOperator, CaseSensitivity, NamespaceConstraint};
use lightningcss::parcel_selectors::matching::{ElementSelectorFlags, MatchingContext};
use lightningcss::parcel_selectors::OpaqueElement;
pub use lightningcss::selector::{PseudoClass, PseudoElement};
use lightningcss::selector::{SelectorIdent, SelectorString, Selectors};
use log::warn;

#[derive(Debug, Default, PartialEq)]
pub(crate) enum ElementName<'i> {
  Pseudo(PseudoElement<'i>),
  Named(&'i str),
  #[default]
  Unnamed,
}

impl<'i> std::fmt::Display for ElementName<'i> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ElementName::Pseudo(e) => e.to_css(f),
      ElementName::Named(s) => s.fmt(f),
      ElementName::Unnamed => Ok(()),
    }
  }
}

/// Object that can be matched against a selector.
#[derive(Default)]
pub struct Element<'i> {
  is_root: bool,
  name: ElementName<'i>,
  pub(crate) id: Option<&'i str>,
  pub(crate) classes: HashSet<&'i str>,
  pseudo_classes: Vec<PseudoClass<'i>>,
  attributes: HashMap<&'i str, &'i str>,
}

impl<'i> std::fmt::Debug for Element<'i> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self)
  }
}

impl<'i> std::fmt::Display for Element<'i> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self.name)?;
    if let Some(id) = &self.id {
      write!(f, "#{}", id)?;
    }
    for class in &self.classes {
      write!(f, ".{}", class)?;
    }
    for class in &self.pseudo_classes {
      class.to_css(f)?;
    }
    for (name, value) in &self.attributes {
      write!(f, "[{}=\"{}\"]", name, value)?;
    }
    Ok(())
  }
}

impl<'i> Element<'i> {
  /// Element with no name.
  pub fn unamed() -> Element<'i> {
    Element::default()
  }

  /// Pseudo element.
  pub fn pseudo(pseudo: PseudoElement<'i>) -> Element<'i> {
    Element {
      name: ElementName::Pseudo(pseudo),
      ..Default::default()
    }
  }

  /// Named element. Matches against: `name {}`.
  pub fn named(name: &'i str) -> Element<'i> {
    Element {
      name: ElementName::Named(name),
      ..Default::default()
    }
  }

  /// `:root` element
  pub fn root() -> Element<'i> {
    Element {
      is_root: true,
      ..Default::default()
    }
  }

  /// `::before` pseudo element.
  pub fn before() -> Element<'i> {
    Element {
      name: ElementName::Pseudo(PseudoElement::Before),
      ..Default::default()
    }
  }

  /// `::after` pseudo element.
  pub fn after() -> Element<'i> {
    Element {
      name: ElementName::Pseudo(PseudoElement::After),
      ..Default::default()
    }
  }

  /// Set element's id. Matches against `#name {}`.
  pub fn id(mut self, id: &'i str) -> Element<'i> {
    self.id = Some(id);
    self
  }

  /// Add attribute to element.
  pub fn attribute(mut self, name: &'i str, value: &'i str) -> Element<'i> {
    self.attributes.insert(name, value);
    self
  }

  /// Add class to element.
  pub fn class(mut self, name: &'i str) -> Element<'i> {
    self.classes.insert(name);
    self
  }

  /// Add pseudo class to element.
  pub fn pseudo_class(mut self, class: PseudoClass<'i>) -> Element<'i> {
    self.pseudo_classes.push(class);
    self
  }
}

impl<'i, 'a> parcel_selectors::Element<'i> for &Element<'a> {
  type Impl = Selectors;

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

  fn has_local_name(&self, local_name: &SelectorIdent<'_>) -> bool {
    if let ElementName::Named(name) = self.name {
      name == local_name.0.as_ref()
    } else {
      false
    }
  }

  fn has_namespace(&self, _ns: &SelectorIdent<'_>) -> bool {
    false
  }

  fn is_same_type(&self, other: &Self) -> bool {
    // Whether this element and the other element have the same local name and
    // namespace.
    other.name == self.name
  }

  fn attr_matches(
    &self,
    _: &NamespaceConstraint<&SelectorIdent<'_>>,
    name: &SelectorIdent<'_>,
    operation: &AttrSelectorOperation<&SelectorString<'_>>,
  ) -> bool {
    let value = self.attributes.get(name.0.as_ref());
    match operation {
      AttrSelectorOperation::Exists => value.is_some(),
      AttrSelectorOperation::WithValue {
        operator,
        case_sensitivity: _,
        expected_value,
      } => {
        let expected_value = expected_value.0.as_ref();
        // See https://developer.mozilla.org/en-US/docs/Web/CSS/Attribute_selectors
        #[allow(clippy::single_match_else)]
        match operator {
          AttrSelectorOperator::Equal => value.map_or(false, |v| *v == expected_value),
          _ => {
            warn!("Unsupported selector");
            false
          },
        }
      },
    }
  }

  // ts == tree-structural (fist-child & such)
  #[allow(clippy::match_same_arms)]
  fn match_non_ts_pseudo_class<F>(&self, pc: &PseudoClass<'i>, _context: &mut MatchingContext<'_, '_, Self::Impl>, _flags_setter: &mut F) -> bool
  where F: FnMut(&Self, ElementSelectorFlags) {
    use PseudoClass::{
      Active, AnyLink, Autofill, Blank, Buffering, Checked, Current, Custom, Default, Defined, Dir, Disabled, Enabled, Focus, FocusVisible, FocusWithin,
      Fullscreen, Future, Hover, InRange, Indeterminate, Invalid, Link, LocalLink, Muted, Optional, OutOfRange, Past, Paused, PlaceholderShown, Playing,
      ReadOnly, ReadWrite, Required, Seeking, Stalled, Target, TargetWithin, UserInvalid, UserValid, Valid, Visited, VolumeLocked, WebKitScrollbar,
    };
    // This exist because we can't use PartialEq (==) between 2 elements of same
    // lifetime.
    self.pseudo_classes.iter().any(|a| {
      match (a, pc) {
        (Hover, Hover) => true,
        (Active, Active) => true,
        (Focus, Focus) => true,
        (FocusVisible, FocusVisible) => true,
        (FocusWithin, FocusWithin) => true,
        (Current, Current) => true,
        (Past, Past) => true,
        (Future, Future) => true,
        (Playing, Playing) => true,
        (Paused, Paused) => true,
        (Seeking, Seeking) => true,
        (Buffering, Buffering) => true,
        (Stalled, Stalled) => true,
        (Muted, Muted) => true,
        (VolumeLocked, VolumeLocked) => true,
        (Defined, Defined) => true,
        (Link, Link) => true,
        (LocalLink, LocalLink) => true,
        (Target, Target) => true,
        (TargetWithin, TargetWithin) => true,
        (Visited, Visited) => true,
        (Enabled, Enabled) => true,
        (Disabled, Disabled) => true,
        (Default, Default) => true,
        (Checked, Checked) => true,
        (Indeterminate, Indeterminate) => true,
        (Blank, Blank) => true,
        (Valid, Valid) => true,
        (Invalid, Invalid) => true,
        (InRange, InRange) => true,
        (OutOfRange, OutOfRange) => true,
        (Required, Required) => true,
        (Optional, Optional) => true,
        (UserValid, UserValid) => true,
        (UserInvalid, UserInvalid) => true,
        (Dir(a), Dir(b)) => a == b,
        (Fullscreen(a), Fullscreen(b)) => a == b,
        (AnyLink(a), AnyLink(b)) => a == b,
        (ReadOnly(a), ReadOnly(b)) => a == b,
        (ReadWrite(a), ReadWrite(b)) => a == b,
        (PlaceholderShown(a), PlaceholderShown(b)) => a == b,
        (Autofill(a), Autofill(b)) => a == b,
        (WebKitScrollbar(a), WebKitScrollbar(b)) => a == b,
        // Local(Box<parcel_selectors::parser::Selector<'i, Selectors>>),
        // Global(Box<parcel_selectors::parser::Selector<'i, Selectors>>),
        // CustomFunction(CowArcStr<'i>, TokenList<'i>),
        // Lang(Vec<CowArcStr<'i>>),
        (Custom(a), Custom(b)) => a == b,
        _ => false,
      }
    })
  }

  #[allow(clippy::match_same_arms)]
  fn match_pseudo_element(&self, pe: &PseudoElement<'i>, _context: &mut MatchingContext<'_, '_, Self::Impl>) -> bool {
    use PseudoElement::{After, Backdrop, Before, Cue, CueRegion, FileSelectorButton, FirstLetter, FirstLine, Marker, Placeholder, Selection, WebKitScrollbar};
    match &self.name {
      // This exist because we can't use PartialEq (==) between 2 elements of same lifetime.
      ElementName::Pseudo(elt) => {
        match (elt, pe) {
          (After, After) => true,
          (Before, Before) => true,
          (FirstLine, FirstLine) => true,
          (FirstLetter, FirstLetter) => true,
          (Cue, Cue) => true,
          (CueRegion, CueRegion) => true,
          (Marker, Marker) => true,
          (Selection(a), Selection(b)) => a == b,
          (Placeholder(a), Placeholder(b)) => a == b,
          (Backdrop(a), Backdrop(b)) => a == b,
          (FileSelectorButton(a), FileSelectorButton(b)) => a == b,
          (WebKitScrollbar(a), WebKitScrollbar(b)) => a == b,
          _ => false,
        }
      },
      _ => false,
    }
  }

  fn is_link(&self) -> bool {
    false
  }

  fn is_html_slot_element(&self) -> bool {
    false
  }

  fn has_id(&self, id: &SelectorIdent<'_>, _: CaseSensitivity) -> bool {
    // Not quirks mode. Always case sensitivie
    self.id.as_ref().map_or(false, |i| *i == id.0.as_ref())
  }

  fn has_class(&self, name: &SelectorIdent<'_>, _: CaseSensitivity) -> bool {
    // Not quirks mode. Always case sensitivie
    self.classes.contains(name.0.as_ref())
  }

  fn imported_part(&self, _name: &SelectorIdent<'_>) -> Option<SelectorIdent<'i>> {
    None
  }

  fn is_part(&self, _name: &SelectorIdent<'_>) -> bool {
    false
  }

  fn is_empty(&self) -> bool {
    true
  }

  fn is_root(&self) -> bool {
    self.is_root
  }
}
