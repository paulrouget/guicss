//! Basic example of Iced integration.

use guicss::iced;
use guicss::integration::iced::{CssEvent, IdAndClasses, SharedRules, CSS};
use iced::subscription::Subscription;
use iced::{Application, Command, Element, Settings};

fn main() -> iced::Result {
  App::run(Settings::default())
}

#[derive(Clone, Debug)]
enum Message {
  ButtonClicked,
  CssEvent(CssEvent),
}

struct App {
  css: CSS,
}

impl Application for App {
  type Executor = iced::executor::Default;
  type Flags = ();
  type Message = Message;
  type Theme = SharedRules;

  fn new(_: Self::Flags) -> (Self, Command<Message>) {
    let path = std::path::PathBuf::from("./examples/iced.css");
    let css = CSS::parse(path).unwrap();
    (App { css }, Command::none())
  }

  fn title(&self) -> String {
    String::from("Iced + guicss")
  }

  fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
    match message {
      Message::CssEvent(CssEvent::Error(e)) => {
        println!("CSS error: {e}");
      },
      Message::CssEvent(CssEvent::Invalidated) => {
        println!("Style has changed");
      },
      Message::ButtonClicked => {
        println!("Button clicked");
      },
    }
    Command::none()
  }

  fn subscription(&self) -> Subscription<Message> {
    self.css.subscription().map(Message::CssEvent)
  }

  fn view(&self) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
    let def = IdAndClasses::parse("#test.foo.bar");
    self.css.button("Foobar", def).on_press(Message::ButtonClicked).into()
  }

  fn theme(&self) -> Self::Theme {
    self.css.rules()
  }
}
