use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
  let path = std::path::PathBuf::from("./examples/basic.css");
  let parser = bgcss::parse(path);

  // let elt = Element::named("hbox");

  let event_loop = EventLoop::new();
  let window = WindowBuilder::new().build(&event_loop).unwrap();

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;
    match event {
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        window_id,
      } if window_id == window.id() => *control_flow = ControlFlow::Exit,
      _ => (),
    }
  });

  // loop {
  //   println!("Waiting parsing event");
  //   let event = parser.thread.recv();
  //   match event {
  //     Err(e) => {
  //       println!("Got error: {:?}", e);
  //       return;
  //     },
  //     Ok(Event::Error(e)) => {
  //       println!("Got error: {:?}", e);
  //     },
  //     Ok(Event::Parsed(rules)) => {
  //       rules.compute(&elt);
  //       println!("Event: Parsed");
  //     },
  //     Ok(event) => {
  //       println!("Event: {:?}", event);
  //     },
  //   }
  // }
}
