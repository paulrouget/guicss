use std::collections::HashMap;

use lightningcss::media_query::{MediaFeature, MediaFeatureValue, Operator, Qualifier};
use lightningcss::parcel_selectors::context::QuirksMode;
use lightningcss::parcel_selectors::matching::{matches_selector, MatchingContext, MatchingMode};
use lightningcss::printer::Printer;
use lightningcss::properties::custom::{CustomProperty, TokenOrValue};
use lightningcss::properties::Property;
use lightningcss::rules::media::MediaRule;
use lightningcss::rules::style::StyleRule;
use lightningcss::rules::CssRule;
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use log::warn;

use crate::elements::Element;
use crate::properties::ComputedProperties;
use crate::theme::Theme;

pub fn compute(stylesheet: &StyleSheet<'_, '_>, element: &Element<'_>, theme: Theme) -> ComputedProperties {
  // Iterator over all the rules, including rules under matching MediaQueries
  let rules_iter = stylesheet.rules.0.iter();
  let mut all_rules: Vec<_> = rules_iter
    .map(|rule| {
      match rule {
        CssRule::Style(style) => [style].to_vec(),
        CssRule::Media(m) => compute_media_queries(m, theme),
        unknown => {
          warn!("Unsupported CSS Rule: {:?}", unknown);
          vec![]
        },
      }
    })
    .flatten()
    .flat_map(|style| style.selectors.0.iter().map(|s| (s, &style.declarations)))
    .collect();

  // Sort all rules by specificity.
  all_rules.sort_by(|(s1, _), (s2, _)| s1.specificity().cmp(&s2.specificity()));

  // Only keep Vec<declarations> of matching rules
  let mut ctx = MatchingContext::new(MatchingMode::Normal, None, None, QuirksMode::NoQuirks);
  let matching: (Vec<_>, Vec<_>) = all_rules
    .into_iter()
    .filter_map(|(s, decs)| {
      if matches_selector(s, 0, None, &element, &mut ctx, &mut |_, _| {}) {
        Some((&decs.declarations, &decs.important_declarations))
      } else {
        None
      }
    })
    .unzip();

  // Flatten the declarations and sort them with important declarations are at the
  // end.
  let (normal_matching, important_matching) = matching;
  let normal_matching = normal_matching.into_iter().flatten();
  let important_matching = important_matching.into_iter().flatten();
  let matching = normal_matching.chain(important_matching);

  // Declarations are not sort from least specific to more specific, with
  // importants at the end.

  let mut variables = HashMap::new();

  // Remove `--x: y` declarations, and save them in hashmap.
  let without_var: Vec<_> = matching
    .filter(|prop| {
      if let Property::Custom(CustomProperty { name, value: tokens }) = prop {
        if name.starts_with("--") {
          let mut source = String::new();
          let mut printer = Printer::new(&mut source, PrinterOptions::default());
          // FIXME: it's unfortunate we have to stringify the already parsed CSS.
          tokens.to_css(&mut printer, false).unwrap();
          variables.insert(name.clone(), source.clone());
          return false;
        }
      }
      true
    })
    .collect();

  let mut computed = ComputedProperties::default();

  // Apply all declarations to computed properties.
  for prop in without_var {
    if let Property::Unparsed(p) = prop {
      let token = p.value.0.get(0);
      if let Some(TokenOrValue::Var(v)) = token {
        let name = v.name.ident.0.as_ref();
        if let Some(source) = variables.get(name) {
          let id = p.property_id.clone();
          if let Ok(prop) = Property::parse_string(id, source, ParserOptions::default()) {
            if let Err(e) = computed.apply(&prop) {
              warn!("{}", e);
            }
            continue;
          } else {
            warn!("Could not parse `{}` variable content ({}) for property {:?}", name, source, prop);
          }
        } else {
          warn!("Could not resolve variable: {}", name);
        }
      }
    }
    if let Err(e) = computed.apply(prop) {
      warn!("{}", e);
    }
  }
  computed
}

fn compute_media_queries<'i, 'a>(media: &'a MediaRule<'i>, theme: Theme) -> Vec<&'a StyleRule<'i>> {
  let matches = media.query.media_queries.iter().any(|m| {
    match m.qualifier {
      Some(Qualifier::Not) => !m.condition.as_ref().map_or(true, |c| does_query_match(c, theme)),
      _ => m.condition.as_ref().map_or(true, |c| does_query_match(c, theme)),
    }
  });
  if matches {
    let rules_iter = media.rules.0.iter();
    // FIXME: only keeping on nesting level of media queries
    rules_iter
      .filter_map(|r| {
        match r {
          CssRule::Style(style) => Some(style),
          _ => None,
        }
      })
      .collect()
  } else {
    vec![]
  }
}

fn does_query_match(condition: &lightningcss::media_query::MediaCondition<'_>, theme: Theme) -> bool {
  use lightningcss::media_query::MediaCondition::{Feature, InParens, Not, Operation};
  match condition {
    Feature(MediaFeature::Plain {
      name,
      value: MediaFeatureValue::Ident(ident),
    }) => {
      match name.as_ref() {
        "os-version" => ident.as_ref() == std::env::consts::OS,
        "prefers-color-scheme" =>
        {
          #[allow(clippy::match_like_matches_macro)]
          match (ident.as_ref(), theme) {
            ("light", Theme::Light) => true,
            ("dark", Theme::Dark) => true,
            _ => false,
          }
        },
        _ => false,
      }
    },
    Not(cond) => !does_query_match(cond, theme),
    Operation(conditions, Operator::And) => conditions.iter().all(|c| does_query_match(c, theme)),
    Operation(conditions, Operator::Or) => conditions.iter().any(|c| does_query_match(c, theme)),
    InParens(condition) => does_query_match(condition, theme),
    _ => {
      // Unsupported
      false
    },
  }
}
