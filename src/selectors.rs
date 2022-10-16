use std::fmt;

use cssparser::{CowRcStr, ParseError, Parser, SourceLocation, ToCss, _cssparser_internal_to_lowercase, match_ignore_ascii_case};
use parcel_selectors::parser::{NestingRequirement, NonTSPseudoClass, PseudoElement as PseudoElementTrait, SelectorImpl, SelectorParseErrorKind};

// FIXME: could this be just a CowRcStr, not wrapped into LocalName?
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SelectorString<'a>(CowRcStr<'a>);

impl<'a> From<CowRcStr<'a>> for SelectorString<'a> {
  fn from(s: CowRcStr<'a>) -> SelectorString<'a> {
    SelectorString(s.clone())
  }
}

impl<'a> ToCss for SelectorString<'a> {
  fn to_css<W>(&self, dest: &mut W) -> fmt::Result
  where W: fmt::Write {
    dest.write_str(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq)]
pub enum PseudoClass {
  Active,
  Hover,
  Focus,
  Enabled,
  Disabled,
}

impl ToCss for PseudoClass {
  fn to_css<W>(&self, dest: &mut W) -> fmt::Result
  where W: fmt::Write {
    dest.write_str(match self {
      PseudoClass::Active => ":active",
      PseudoClass::Hover => ":hover",
      PseudoClass::Focus => ":focus",
      PseudoClass::Enabled => ":enabled",
      PseudoClass::Disabled => ":disabled",
    })
  }
}

impl NonTSPseudoClass<'_> for PseudoClass {
  type Impl = CustomParser;

  fn is_active_or_hover(&self) -> bool {
    matches!(self, PseudoClass::Active | PseudoClass::Hover)
  }

  fn is_user_action_state(&self) -> bool {
    true
  }
}

#[derive(Clone, PartialEq, Eq)]
pub enum PseudoElement {
  Scrollbar,
}

impl ToCss for PseudoElement {
  fn to_css<W>(&self, dest: &mut W) -> fmt::Result
  where W: fmt::Write {
    dest.write_str(match self {
      PseudoElement::Scrollbar => "::scrollbar",
    })
  }
}

impl PseudoElementTrait<'_> for PseudoElement {
  type Impl = CustomParser;
}

pub(crate) type SelectorList<'i> = parcel_selectors::SelectorList<'i, CustomParser>;
pub(crate) type Selector<'i> = parcel_selectors::parser::Selector<'i, CustomParser>;

impl<'i> SelectorImpl<'i> for CustomParser {
  type AttrValue = SelectorString<'i>;
  type BorrowedLocalName = SelectorString<'i>;
  type BorrowedNamespaceUrl = SelectorString<'i>;
  type ExtraMatchingData = ();
  type Identifier = SelectorString<'i>;
  type LocalName = SelectorString<'i>;
  type NamespacePrefix = SelectorString<'i>;
  type NamespaceUrl = SelectorString<'i>;
  type NonTSPseudoClass = PseudoClass;
  type PseudoElement = PseudoElement;
  type VendorPrefix = SelectorString<'i>;
}

#[derive(Clone, Debug)]
pub struct CustomParser;

pub(crate) fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<SelectorList<'i>, ParseError<'i, SelectorParseErrorKind<'i>>> {
  let parser = CustomParser;
  let reqs = NestingRequirement::None;
  SelectorList::parse(&parser, input, reqs)
}

impl<'i> parcel_selectors::parser::Parser<'i> for CustomParser {
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
