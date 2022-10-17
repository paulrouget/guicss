use std::fmt;

use crate::elements::{PseudoElement, PseudoClass};

use cssparser::{CowRcStr, ParseError, Parser, SourceLocation, ToCss, _cssparser_internal_to_lowercase, match_ignore_ascii_case};
use selectors::parser::{NonTSPseudoClass, PseudoElement as PseudoElementTrait, SelectorImpl, SelectorParseErrorKind};

// FIXME: could this be just a CowRcStr, not wrapped into LocalName?
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SelectorString(String);

impl<'a> AsRef<str> for SelectorString {
  fn as_ref(&self) -> &str {
    self.0.as_ref()
  }
}

impl From<&str> for SelectorString {
  fn from(s: &str) -> SelectorString {
    SelectorString(s.to_string())
  }
}

impl ToCss for SelectorString {
  fn to_css<W>(&self, dest: &mut W) -> fmt::Result
  where W: fmt::Write {
    dest.write_str(&self.0)
  }
}

impl ToCss for PseudoClass {
  fn to_css<W>(&self, f: &mut W) -> fmt::Result
  where W: fmt::Write {
    write!(f, "{}", &self)
  }
}

impl ToCss for PseudoElement {
  fn to_css<W>(&self, f: &mut W) -> fmt::Result
  where W: fmt::Write {
    write!(f, "{}", &self)
  }
}

impl NonTSPseudoClass for PseudoClass {
  type Impl = CustomParser;

  fn is_active_or_hover(&self) -> bool {
    matches!(self, PseudoClass::Active | PseudoClass::Hover)
  }

  fn is_user_action_state(&self) -> bool {
    true
  }
}

impl PseudoElementTrait for PseudoElement {
  type Impl = CustomParser;
}

pub(crate) type SelectorList = selectors::SelectorList<CustomParser>;
pub(crate) type Selector = selectors::parser::Selector<CustomParser>;

impl<'i> SelectorImpl for CustomParser {
  type AttrValue = SelectorString;
  type BorrowedLocalName = SelectorString;
  type BorrowedNamespaceUrl = SelectorString;
  type ExtraMatchingData = ();
  type Identifier = SelectorString;
  type LocalName = SelectorString;
  type NamespacePrefix = SelectorString;
  type NamespaceUrl = SelectorString;
  type NonTSPseudoClass = PseudoClass;
  type PseudoElement = PseudoElement;
}

#[derive(Clone, Debug)]
pub struct CustomParser;

pub(crate) fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<SelectorList, ParseError<'i, SelectorParseErrorKind<'i>>> {
  let parser = CustomParser;
  SelectorList::parse(&parser, input)
}

impl<'i> selectors::parser::Parser<'i> for CustomParser {
  type Error = SelectorParseErrorKind<'i>;
  type Impl = CustomParser;

  fn parse_non_ts_pseudo_class(&self, loc: SourceLocation, name: CowRcStr<'i>) -> Result<PseudoClass, ParseError<'i, Self::Error>> {
    match_ignore_ascii_case! { &name,
      "hover" => Ok(PseudoClass::Hover),
      "active" => Ok(PseudoClass::Active),
      "focus" => Ok(PseudoClass::Focus),
      "enabled" => Ok(PseudoClass::Enabled),
      "disabled" => Ok(PseudoClass::Disabled),
      _ => Err(loc.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name.clone())))
    }
  }

  fn parse_pseudo_element(&self, loc: SourceLocation, name: CowRcStr<'i>) -> Result<PseudoElement, ParseError<'i, Self::Error>> {
    match_ignore_ascii_case! { &name,
      "scrollbar" => Ok(PseudoElement::Scrollbar),
      _ => Err(loc.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name.clone())))
    }
  }
}
