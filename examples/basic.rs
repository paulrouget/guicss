use bgcss::elements::Element;
use bgcss::theme::get_theme;
use bgcss::thread::{spawn_and_parse, Event};
use log::error;
use winit::event::{Event as WinitEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopBuilder};

fn main() {
  let event_loop = EventLoopBuilder::with_user_event().build();
  let proxy = event_loop.create_proxy();

  let path = std::path::PathBuf::from("./examples/basic.css");
  let elt = Element::named("hbox").id("foo");

  spawn_and_parse(path, move |event| {
    if let Err(e) = proxy.send_event(event) {
      error!("Sending user event failed: {}", e);
    }
  });

  let mut rules = None;

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;
    match event {
      WinitEvent::UserEvent(event) => {
        match event {
          Event::Error(e) => {
            println!("Got error: {:?}", e);
          },
          Event::Parsed(new_rules) => {
            println!("Event: Parsed");
            let theme = get_theme();
            let c = new_rules.compute(&elt, theme);
            println!("Computed: {:?}", c);
            rules = Some(new_rules);
          },
          Event::ThemeChanged => {
            if let Some(rules) = rules.as_ref() {
              let theme = get_theme();
              let c = rules.compute(&elt, theme);
              println!("Computed: {:?}", c);
            }
          },
          event => {
            println!("Event: {:?}", event);
          },
        }
      },
      WinitEvent::WindowEvent {
        event: WindowEvent::CloseRequested,
        window_id: _,
      } => *control_flow = ControlFlow::Exit,
      _ => (),
    }
  });
}
