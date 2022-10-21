use log::error;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopBuilder};


fn main() {
  let event_loop = EventLoopBuilder::with_user_event().build();
  let proxy = event_loop.create_proxy();

  let path = std::path::PathBuf::from("./examples/basic.css");
  let elt = bgcss::Element::named("hbox");

  bgcss::parse(path, move |event| {
    if let Err(e) = proxy.send_event(event) {
      error!("Sending user event failed: {}", e);
    }
  });

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;
    match event {
      Event::UserEvent(event) => {
        match event {
          bgcss::Event::Error(e) => {
            println!("Got error: {:?}", e);
          },
          bgcss::Event::Parsed(rules) => {
            rules.compute(&elt);
            println!("Event: Parsed");
          },
          event => {
            println!("Event: {:?}", event);
          },
        }
      },
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        window_id: _,
      } => *control_flow = ControlFlow::Exit,
      _ => (),
    }
  });
}
