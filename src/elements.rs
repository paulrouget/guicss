use parcel_selectors::OpaqueElement;
use parcel_selectors::attr::*;
use parcel_selectors::matching::MatchingContext;
use parcel_selectors::matching::ElementSelectorFlags;

use crate::selectors::SelectorStr;

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

#[derive(Debug)]
enum ElementType<'i> {
  Pseudo(PseudoElement),
  Regular(&'i str),
}

#[derive(Debug)]
struct Element<'i> {
  r#type: ElementType<'i>,
  id: Option<&'i str>,
  classes: Vec<&'i str>,
  pseudo_classes: Vec<PseudoClass>,
  attributes: Vec<(&'i str, &'i str)>,
}

type Node<'i> = indextree::Node<Element<'i>>;

#[derive(Clone)]
struct ElementRef<'i> {
  id: indextree::NodeId,
  arena: &'i indextree::Arena<Element<'i>>,
}

impl<'i> std::fmt::Debug for ElementRef<'i> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "ElementRef NodeId: {}", self.id)
  }
}

impl<'i> ElementRef<'i> {
  fn new(&self, id: indextree::NodeId) -> Self {
    ElementRef { id, arena: self.arena }
  }
  fn node(&self) -> &Node {
    self.arena.get(self.id).as_ref().unwrap()
  }
}

impl<'i> parcel_selectors::Element<'i> for ElementRef<'i> {
  type Impl = crate::selectors::CustomParser;
  fn opaque(&self) -> OpaqueElement {
    OpaqueElement::new(&self)
  }
  fn parent_element(&self) -> Option<Self> {
    self.node().parent().map(|id| self.new(id))
  }
  fn parent_node_is_shadow_root(&self) -> bool {
    false
  }
  fn containing_shadow_host(&self) -> Option<Self> {
    None
  }
  fn is_pseudo_element(&self) -> bool {
    matches!(self.node().get().r#type, ElementType::Pseudo(_))
  }
  fn prev_sibling_element(&self) -> Option<Self> {
    self.node().previous_sibling().map(|id| self.new(id))
  }
  fn next_sibling_element(&self) -> Option<Self> {
    self.node().next_sibling().map(|id| self.new(id))
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
