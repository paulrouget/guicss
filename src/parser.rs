use cssparser::{
  AtRuleParser, CowRcStr, DeclarationListParser, DeclarationParser, ParseError, Parser, ParserInput, ParserState, QualifiedRuleParser, RuleListParser,
  _cssparser_internal_to_lowercase, match_ignore_ascii_case, parse_important,
};
use selectors::context::QuirksMode;
use selectors::matching::{matches_selector, MatchingContext, MatchingMode};
use selectors::parser::SelectorParseErrorKind;

use crate::elements::Element;
use crate::properties::{ComputedProperties, Importance, Property};
use crate::selectors::{Selector, SelectorList};

struct CustomParser;
struct CustomDecParser;

#[derive(Debug)]
pub(crate) enum CustomError<'src> {
  UnknownProperty(CowRcStr<'src>),
  SelectorError(SelectorParseErrorKind<'src>),
}

impl<'src> From<SelectorParseErrorKind<'src>> for CustomError<'src> {
  fn from(e: SelectorParseErrorKind<'src>) -> CustomError<'src> {
    CustomError::SelectorError(e)
  }
}

pub(crate) struct ErrorFormatter<'a, 'b, 'c>(pub &'a ParseError<'b, CustomError<'c>>);

impl std::fmt::Display for ErrorFormatter<'_, '_, '_> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    use cssparser::{BasicParseErrorKind, ParseErrorKind, ToCss};
    use selectors::parser::SelectorParseErrorKind as SPEK;

    write!(f, "[{}:{}]: ", self.0.location.line, self.0.location.column)?;

    match &self.0.kind {
      ParseErrorKind::Basic(e) => {
        match e {
          BasicParseErrorKind::UnexpectedToken(token) => {
            write!(f, "unexpected token: ")?;
            token.to_css(f)
          },
          BasicParseErrorKind::EndOfInput => write!(f, "unexpected end of input"),
          BasicParseErrorKind::AtRuleInvalid(s) => write!(f, "invalid AtRule: {}", s),
          BasicParseErrorKind::AtRuleBodyInvalid => write!(f, "invalid AtRule body"),
          BasicParseErrorKind::QualifiedRuleInvalid => write!(f, "qualified rule invalid"),
        }
      },
      ParseErrorKind::Custom(e) => {
        match e {
          CustomError::UnknownProperty(s) => write!(f, "unknown property: {}", s),
          CustomError::SelectorError(e) => {
            match e {
              SPEK::UnsupportedPseudoClassOrElement(s) => write!(f, "unsupported pseudoClass or pseudoElement: {}", s),
              SPEK::UnexpectedIdent(s) => write!(f, "unexpected identifier: {}", s),
              SPEK::ExpectedNamespace(s) => write!(f, "expected namespace: {}", s),
              SPEK::NonCompoundSelector => write!(f, "non compound selector"),
              SPEK::NonPseudoElementAfterSlotted => write!(f, "non pseudoElement after slotted"),
              SPEK::InvalidPseudoElementAfterSlotted => write!(f, "invalid pseudoElement after slotted"),
              SPEK::InvalidPseudoElementInsideWhere => write!(f, "invalid pseudoElement inside where"),
              SPEK::NoQualifiedNameInAttributeSelector(token) => {
                write!(f, "no qualified name in attribut selector")?;
                token.to_css(f)
              },
              SPEK::PseudoElementExpectedColon(token) => {
                write!(f, "pseudoElement expected colon")?;
                token.to_css(f)
              },
              SPEK::NoIdentForPseudo(token) => {
                write!(f, "No identifier for pseudo")?;
                token.to_css(f)
              },
              SPEK::EmptySelector => write!(f, "empty selector"),
              SPEK::DanglingCombinator => write!(f, "dangling combinator"),
              SPEK::InvalidState => write!(f, "invalid state"),
              SPEK::UnexpectedTokenInAttributeSelector(token) => {
                write!(f, "unexpected token in attribute selector")?;
                token.to_css(f)
              },
              SPEK::PseudoElementExpectedIdent(token) => {
                write!(f, "pseudoElement expected")?;
                token.to_css(f)
              },
              SPEK::ExpectedBarInAttr(token) => {
                write!(f, "expected bar in attribute")?;
                token.to_css(f)
              },
              SPEK::BadValueInAttr(token) => {
                write!(f, "bad value in attribute")?;
                token.to_css(f)
              },
              SPEK::InvalidQualNameInAttr(token) => {
                write!(f, "invalid qualified name in attribute")?;
                token.to_css(f)
              },
              SPEK::ExplicitNamespaceUnexpectedToken(token) => {
                write!(f, "explicit namespace unexpected token")?;
                token.to_css(f)
              },
              SPEK::ClassNeedsIdent(token) => {
                write!(f, "class needs identifier")?;
                token.to_css(f)
              },
            }
          },
        }
      },
    }
  }
}

pub(crate) struct ParseResult<'src> {
  pub(crate) rules: Rules,
  pub(crate) errors: Vec<ParseError<'src, CustomError<'src>>>,
}

#[derive(Debug)]
pub struct Rule {
  selector: Selector,
  properties: Vec<(Property, Importance)>,
}

#[derive(Debug)]
pub struct Rules(Vec<Rule>);

impl Rules {
  pub fn compute(&self, element: &Element) -> ComputedProperties {
    let mut context = MatchingContext::new(MatchingMode::Normal, None, None, QuirksMode::NoQuirks);
    let mut computed = ComputedProperties::default();
    let mut delayed: Vec<&Property> = Vec::new();
    for rule in &self.0 {
      let matches = matches_selector(&rule.selector, 0, None, &element, &mut context, &mut |_, _| {});
      if matches {
        let importants = rule.properties.iter().filter(|(_, i)| i.is_important()).map(|(p, _)| p);
        delayed.extend(importants);
        let props = rule.properties.iter().map(|(p, _)| p);
        computed.import(props);
      }
    }
    computed.import(delayed);
    computed
  }
}

pub(crate) fn parse(source: &'_ str) -> ParseResult<'_> {
  let mut parse_input = ParserInput::new(source);
  let mut parser = Parser::new(&mut parse_input);

  let mut errors = Vec::new();
  let mut rules = Vec::new();

  RuleListParser::new_for_stylesheet(&mut parser, CustomParser).for_each(|res| {
    match res {
      Err((e, _)) => {
        errors.push(e);
      },
      Ok(mut res) => {
        errors.append(&mut res.errors);
        rules.append(&mut res.rules.0);
      },
    }
  });

  rules.sort_by(|a, b| a.selector.specificity().cmp(&b.selector.specificity()));

  ParseResult { rules: Rules(rules), errors }
}

impl<'src> QualifiedRuleParser<'src> for CustomParser {
  type Error = CustomError<'src>;
  type Prelude = SelectorList;
  type QualifiedRule = ParseResult<'src>;

  fn parse_prelude<'t>(&mut self, input: &mut Parser<'src, 't>) -> Result<Self::Prelude, ParseError<'src, Self::Error>> {
    crate::selectors::parse(input).map_err(|e| e.into())
  }

  fn parse_block<'t>(
    &mut self,
    prelude: Self::Prelude,
    _: &ParserState,
    input: &mut Parser<'src, 't>,
  ) -> Result<Self::QualifiedRule, ParseError<'src, Self::Error>> {
    let mut decs = Vec::new();
    let mut errors = Vec::new();
    for dec in DeclarationListParser::new(input, CustomDecParser) {
      match dec {
        Ok(d) => {
          decs.push(d);
        },
        Err((e, _)) => {
          errors.push(e);
        },
      }
    }
    let rules = prelude
      .0
      .into_iter()
      .map(|s| {
        Rule {
          selector: s,
          properties: decs.clone(),
        }
      })
      .collect();
    Ok(ParseResult { rules: Rules(rules), errors })
  }
}

impl<'src> DeclarationParser<'src> for CustomDecParser {
  type Declaration = (Property, Importance);
  type Error = CustomError<'src>;

  fn parse_value<'t>(&mut self, name: CowRcStr<'src>, input: &mut Parser<'src, 't>) -> Result<Self::Declaration, ParseError<'src, Self::Error>> {
    let prop = match_ignore_ascii_case! { &name,
      "padding-top" => input.expect_number().map(Property::PaddingTop).map_err(|e| e.into()),
      "padding-bottom" => input.expect_number().map(Property::PaddingBottom).map_err(|e| e.into()),
      "padding-left" => input.expect_number().map(Property::PaddingLeft).map_err(|e| e.into()),
      "padding-right" => input.expect_number().map(Property::PaddingRight).map_err(|e| e.into()),
      "margin-top" => input.expect_number().map(Property::MarginTop).map_err(|e| e.into()),
      "margin-bottom" => input.expect_number().map(Property::MarginBottom).map_err(|e| e.into()),
      "margin-left" => input.expect_number().map(Property::MarginLeft).map_err(|e| e.into()),
      "margin-right" => input.expect_number().map(Property::MarginRight).map_err(|e| e.into()),
      _ => Err(input.new_custom_error(CustomError::UnknownProperty(name.clone()))),
    };
    let prop = prop.map(|p| {
      let importance = match input.try_parse(parse_important) {
        Ok(_) => Importance::Important,
        Err(_) => Importance::NotImportant,
      };
      (p, importance)
    });
    input.expect_exhausted()?;
    prop
  }
}

// Unsupported: FIXME support dark/light

impl<'src> AtRuleParser<'src> for CustomParser {
  type AtRule = ParseResult<'src>;
  type Error = CustomError<'src>;
  type Prelude = ();
}

impl<'src> AtRuleParser<'src> for CustomDecParser {
  type AtRule = (Property, Importance);
  type Error = CustomError<'src>;
  type Prelude = ();
}
