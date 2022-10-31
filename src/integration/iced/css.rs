use std::cell::Cell;
use std::path::PathBuf;

use anyhow::Result;
use iced::futures::channel::mpsc;
use iced::futures::{FutureExt, SinkExt, StreamExt, TryFutureExt};
use iced::subscription::{self, Subscription};

use crate::element::Element;
use crate::integration::iced::shared_rules::SharedRules;
use crate::integration::iced::{CssEvent, IdAndClasses};
use crate::parser::{parse_file, parse_file_sync, Event};

pub struct CSS {
  pub(crate) rules: SharedRules,
  receiver: Cell<Option<mpsc::UnboundedReceiver<CssEvent>>>,
}

impl CSS {
  pub fn parse(path: PathBuf) -> Result<CSS> {
    let rules = parse_file_sync(&path)?;
    let rules = SharedRules::new(rules);
    let shared = rules.clone();
    let (mut sender, receiver) = mpsc::unbounded();
    parse_file(path, move |event| {
      match event {
        Event::Error(e) => sender.send(CssEvent::Error(e)),
        Event::Invalidated(rules) => {
          shared.update(rules);
          sender.send(CssEvent::Invalidated)
        },
      }
      .unwrap_or_else(|e| eprintln!("Could not send event {e:?}"))
      .now_or_never();
    });
    let receiver = Cell::new(Some(receiver));
    Ok(CSS { rules, receiver })
  }

  pub fn rules(&self) -> SharedRules {
    // FIXME: does that happen often?
    self.rules.clone()
  }

  pub fn subscription(&self) -> Subscription<CssEvent> {
    let receiver = self.receiver.replace(None);
    struct Sub;
    subscription::unfold(std::any::TypeId::of::<Sub>(), receiver, |mut receiver| {
      async move {
        let event = receiver.as_mut().unwrap().select_next_some().await;
        (Some(event), receiver)
      }
    })
  }
}
