use cssparser::{
  AtRuleParser, CowRcStr, DeclarationListParser, DeclarationParser, ParseError, Parser, ParserInput, ParserState, QualifiedRuleParser, RuleListParser,
  _cssparser_internal_to_lowercase, match_ignore_ascii_case, parse_important,
};

use parcel_selectors::parser::SelectorParseErrorKind;
use crate::properties::{Importance, Property};
use crate::selectors::{Selector, SelectorList};

struct CustomParser;
struct CustomDecParser;

#[derive(Debug)]
pub(crate) enum CustomError<'i> {
  UnknownProperty(CowRcStr<'i>),
  SelectorError(SelectorParseErrorKind<'i>),
}

impl<'i> From<SelectorParseErrorKind<'i>> for CustomError<'i> {
  fn from(e: SelectorParseErrorKind<'i>) -> CustomError<'i> {
    CustomError::SelectorError(e)
  }
}

pub(crate) struct WithErrors<'i, T> {
  pub(crate) inner: T,
  pub(crate) errors: Vec<ParseError<'i, CustomError<'i>>>,
}

pub(crate) struct Rule<'i> {
  selector: Selector<'i>,
  properties: Vec<(Property, Importance)>,
}

impl<'i> Rule<'i> {
  // pub fn matches(&self, element: &E) -> bool where E: Element<'a> {
  //   let mut context = MatchingContext::new(MatchingMode::Normal, None, None, QuirksMode::NoQuirks);
  //   matches_selector(&self.selector, 0, None, element, &mut context, &mut |_, _| {})
  //   // todo!()
  // }
}

pub(crate) fn parse(source: &str) -> WithErrors<'_, Vec<Rule<'_>>> {
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
        rules.append(&mut res.inner);
      },
    }
  });

  rules.sort_by(|a, b| a.selector.specificity().cmp(&b.selector.specificity()));

  WithErrors { inner: rules, errors }
}

impl<'i> QualifiedRuleParser<'i> for CustomParser {
  type Error = CustomError<'i>;
  type Prelude = SelectorList<'i>;
  type QualifiedRule = WithErrors<'i, Vec<Rule<'i>>>;

  fn parse_prelude<'t>(&mut self, input: &mut Parser<'i, 't>) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
    crate::selectors::parse(input).map_err(|e| e.into())
  }

  fn parse_block<'t>(
    &mut self,
    prelude: Self::Prelude,
    _: &ParserState,
    input: &mut Parser<'i, 't>,
  ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
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
    Ok(WithErrors { inner: rules, errors })
  }
}

impl<'i> DeclarationParser<'i> for CustomDecParser {
  type Declaration = (Property, Importance);
  type Error = CustomError<'i>;

  fn parse_value<'t>(&mut self, name: CowRcStr<'i>, input: &mut Parser<'i, 't>) -> Result<Self::Declaration, ParseError<'i, Self::Error>> {
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

impl<'i> AtRuleParser<'i> for CustomParser {
  type AtRule = WithErrors<'i, Vec<Rule<'i>>>;
  type Error = CustomError<'i>;
  type Prelude = ();
}

impl<'i> AtRuleParser<'i> for CustomDecParser {
  type AtRule = (Property, Importance);
  type Error = CustomError<'i>;
  type Prelude = ();
}
