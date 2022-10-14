//! Basic example. Forwarding `guicss` events to Winit event loop,

use winit::event::{Event as WinitEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopBuilder};

fn main() {
  let event_loop = EventLoopBuilder::with_user_event().build();
  let proxy = event_loop.create_proxy();

  let path = std::path::PathBuf::from("./examples/basic.css");
  let elt = guicss::Element::named("hbox").id("foo");

  guicss::parse_file(path, move |event| {
    if let Err(e) = proxy.send_event(event) {
      eprintln!("Sending user event failed: {}", e);
    }
  });

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;
    match event {
      WinitEvent::UserEvent(event) => {
        match event {
          guicss::Event::Error(e) => {
            eprintln!("Got error: {:?}", e);
          },
          guicss::Event::Invalidated(new_rules) => {
            println!("Event: Parsed");
            let c = new_rules.compute(&elt);
            println!("Computed: {:?}", c);
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
