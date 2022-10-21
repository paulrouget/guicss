use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};

use log::error;

fn main() {
  let path = std::path::PathBuf::from("./examples/basic.css");
  let parser = bgcss::parse(path);

  let elt = bgcss::Element::named("hbox");

  let event_loop = EventLoopBuilder::with_user_event().build();
  let window = WindowBuilder::new().build(&event_loop).unwrap();

  let proxy: winit::event_loop::EventLoopProxy<bgcss::Event> = event_loop.create_proxy();

  std::thread::spawn(move || {
    loop {
      if let Err(e) = match parser.recv() {
        Ok(evt) => proxy.send_event(evt),
        Err(e) => proxy.send_event(bgcss::Event::Error(e.to_string())),
      } {
        error!("Sending user event failed: {}", e);
      }
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
        window_id,
      } if window_id == window.id() => *control_flow = ControlFlow::Exit,
      _ => (),
    }
  });

}
