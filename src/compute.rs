use std::collections::HashMap;

use lightningcss::declaration::DeclarationBlock;
use lightningcss::media_query::{MediaFeature, MediaFeatureValue, MediaQuery, Operator, Qualifier};
use lightningcss::parcel_selectors::context::QuirksMode;
use lightningcss::parcel_selectors::matching::{matches_selector, MatchingContext, MatchingMode};
use lightningcss::parcel_selectors::parser::Selector;
use lightningcss::printer::Printer;
use lightningcss::properties::custom::{CustomProperty, TokenOrValue, UnparsedProperty, Variable};
use lightningcss::properties::Property;
use lightningcss::rules::media::MediaRule;
use lightningcss::rules::CssRule;
use lightningcss::selector::Selectors;
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::values::ident::{DashedIdent, DashedIdentReference};

use crate::elements::Element;
use crate::properties::ComputedProperties;

pub fn compute(stylesheet: &StyleSheet<'_, '_>, element: &Element<'_>) -> ComputedProperties {
  let mut variables = HashMap::new();
  let mut computed = ComputedProperties::default();
  let mut rules: Vec<(&Selector<'_, Selectors>, &DeclarationBlock<'_>)> = stylesheet
    .rules
    .0
    .iter()
    .filter_map(|rule| {
      match rule {
        CssRule::Style(style) => Some([style].to_vec()),
        CssRule::Media(MediaRule { query, rules, .. }) => {
          let matches = query.media_queries.iter().any(
            |MediaQuery {
               qualifier,
               media_type: _,
               condition,
             }| {
              match qualifier {
                Some(Qualifier::Not) => !condition.as_ref().map_or(true, check_media_query),
                _ => condition.as_ref().map_or(true, check_media_query),
              }
            },
          );
          if matches {
            // FIXME: only keeping on nesting level of media queries
            Some(
              rules
                .0
                .iter()
                .filter_map(|r| {
                  match r {
                    CssRule::Style(style) => Some(style),
                    _ => None,
                  }
                })
                .collect(),
            )
          } else {
            None
          }
        },
        unknown => {
          println!("Unsupported: {:?}", unknown);
          None
        },
      }
    })
    .flatten()
    .flat_map(|style| style.selectors.0.iter().map(|s| (s, &style.declarations)))
    .collect();
  rules.sort_by(|(s1, _), (s2, _)| s1.specificity().cmp(&s2.specificity()));

  let mut context = MatchingContext::new(MatchingMode::Normal, None, None, QuirksMode::NoQuirks);
  let (normal, important): (Vec<_>, Vec<_>) = rules
    .into_iter()
    .filter_map(|(s, decs)| {
      if matches_selector(s, 0, None, &element, &mut context, &mut |_, _| {}) {
        Some((&decs.declarations, &decs.important_declarations))
      } else {
        None
      }
    })
    .unzip();

  let normal = normal.into_iter().flatten();
  let important = important.into_iter().flatten();

  let all = normal.chain(important);

  let without_var: Vec<_> = all
    .filter(|prop| {
      if let Property::Custom(CustomProperty { name, value: tokens }) = prop {
        if name.starts_with("--") {
          let mut source = String::new();
          let mut printer = Printer::new(&mut source, PrinterOptions::default());
          tokens.to_css(&mut printer, false).unwrap();
          variables.insert(name.clone(), source.clone());
          return false;
        }
      }
      true
    })
    .collect();

  for prop in without_var {
    if let Property::Unparsed(UnparsedProperty {
      property_id: id,
      value: tokens,
    }) = prop
    {
      if let Some(TokenOrValue::Var(Variable {
        name: DashedIdentReference { ident: DashedIdent(name), .. },
        ..
      })) = tokens.0.get(0)
      {
        if let Some(source) = variables.get(name) {
          if let Ok(prop) = Property::parse_string(id.clone(), source, ParserOptions::default()) {
            if let Err(e) = computed.apply(&prop) {
              eprintln!("{}", e);
            }
            continue;
          } else {
            eprintln!("Could not parse `{}` variable content ({}) for property {:?}", name, source, prop);
          }
        } else {
          eprintln!("Could not resolve variable: {}", name);
        }
      }
    }
    if let Err(e) = computed.apply(prop) {
      eprintln!("{}", e);
    }
  }
  computed
}

fn check_media_query(condition: &lightningcss::media_query::MediaCondition<'_>) -> bool {
  use lightningcss::media_query::MediaCondition::{Feature, InParens, Not, Operation};
  match condition {
    Feature(MediaFeature::Plain {
      name,
      value: MediaFeatureValue::Ident(ident),
    }) => {
      match name.as_ref() {
        "os-version" => ident.as_ref() == std::env::consts::OS,
        "prefers-color-scheme" => ident.as_ref() == "light", // FIXME
        _ => false,
      }
    },
    Not(cond) => !check_media_query(cond),
    Operation(conditions, Operator::And) => conditions.iter().all(check_media_query),
    Operation(conditions, Operator::Or) => conditions.iter().any(check_media_query),
    InParens(condition) => check_media_query(condition),
    _ => {
      // Unsupported
      false
    },
  }
}
