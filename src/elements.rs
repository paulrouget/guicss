use parcel_selectors::OpaqueElement;
use parcel_selectors::attr::*;
use parcel_selectors::matching::MatchingContext;
use parcel_selectors::matching::ElementSelectorFlags;

// FIXME: sad we have to use both NodeId and NodeRef

use crate::selectors::SelectorStr;

pub type Arena<'i> = indextree::Arena<Element<'i>>;
pub use indextree::NodeId;

type Node<'i> = indextree::Node<Element<'i>>;

#[derive(Clone)]
pub struct NodeRef<'i> {
  id: NodeId,
  arena: &'i Arena<'i>,
}

impl<'i> NodeRef<'i> {
  fn new(id: indextree::NodeId, arena: &'i Arena) -> Self {
    NodeRef { id, arena }
  }
  fn dupe(&self, id: indextree::NodeId) -> Self {
    NodeRef { id, arena: self.arena }
  }
  fn node(&self) -> &Node {
    self.arena.get(self.id).as_ref().unwrap()
  }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PseudoClass {
  Active,
  Hover,
  Focus,
  Enabled,
  Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PseudoElement {
  Scrollbar,
}

#[derive(Debug, Default)]
pub enum ElementName<'i> {
  Pseudo(PseudoElement),
  Named(&'i str),
  #[default]
  Unnamed,
}

#[derive(Debug, Default)]
pub struct Element<'i> {
  name: ElementName<'i>,
  id: Option<&'i str>,
  classes: Vec<&'i str>,
  pseudo_classes: Vec<PseudoClass>,
  attributes: Vec<(&'i str, &'i str)>,
}

impl<'i> Element<'i> {
  pub fn new() -> Element<'i> {
    Element::default()
  }
  pub fn name(mut self, name: &'i str) -> Element {
    self.name = ElementName::Named(name);
    self
  }
  pub fn id(mut self, id: &'i str) -> Element {
    self.id = Some(id);
    self
  }
}

impl<'i> std::fmt::Debug for NodeRef<'i> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "NodeRef NodeId: {}", self.id)
  }
}

impl<'i> parcel_selectors::Element<'i> for NodeRef<'i> {
  type Impl = crate::selectors::CustomParser;
  fn opaque(&self) -> OpaqueElement {
    OpaqueElement::new(&self)
  }
  fn parent_element(&self) -> Option<Self> {
    self.node().parent().map(|id| self.dupe(id))
  }
  fn parent_node_is_shadow_root(&self) -> bool {
    false
  }
  fn containing_shadow_host(&self) -> Option<Self> {
    None
  }
  fn is_pseudo_element(&self) -> bool {
    matches!(self.node().get().name, ElementName::Pseudo(_))
  }
  fn prev_sibling_element(&self) -> Option<Self> {
    self.node().previous_sibling().map(|id| self.dupe(id))
  }
  fn next_sibling_element(&self) -> Option<Self> {
    self.node().next_sibling().map(|id| self.dupe(id))
  }
  fn is_html_element_in_html_document(&self) -> bool {
    false
  }
  fn has_local_name(&self, local_name: &SelectorStr) -> bool {
    todo!()
  }

  fn has_namespace(&self, ns: &SelectorStr) -> bool {
    todo!()
  }

  fn is_same_type(&self, other: &Self) -> bool {
    todo!()
  }

  fn attr_matches(
    &self,
    ns: &NamespaceConstraint<&SelectorStr>,
    local_name: &SelectorStr,
    operation: &AttrSelectorOperation<&SelectorStr>
    ) -> bool {
    todo!()
  }
  fn match_non_ts_pseudo_class<F>(
    &self,
    pc: &PseudoClass,
    context: &mut MatchingContext<'_, 'i, Self::Impl>,
    flags_setter: &mut F
    ) -> bool
    where
      F: FnMut(&Self, ElementSelectorFlags) {
        todo!()
      }
  fn match_pseudo_element(
    &self,
    pe: &PseudoElement,
    context: &mut MatchingContext<'_, 'i, Self::Impl>
    ) -> bool {
    todo!()
  }
  fn is_link(&self) -> bool {
    false
  }
  fn is_html_slot_element(&self) -> bool {
    false
  }
  fn has_id(
    &self,
    id: &SelectorStr,
    case_sensitivity: CaseSensitivity
    ) -> bool {
    self.node().get().id.is_some()
  }
  fn has_class(
    &self,
    name: &SelectorStr,
    case_sensitivity: CaseSensitivity
    ) -> bool {
    todo!()
  }
  fn imported_part( &self, name: &SelectorStr) -> Option<SelectorStr<'i>> {
    None
  }
  fn is_part(&self, name: &SelectorStr) -> bool {
    false
  }
  fn is_empty(&self) -> bool {
    todo!()
  }
  fn is_root(&self) -> bool {
    todo!()
  }

  // fn pseudo_element_originating_element(&self) -> Option<Self> { ... }
  // fn assigned_slot(&self) -> Option<Self> { ... }
  // fn ignores_nth_child_selectors(&self) -> bool { ... } 
}
