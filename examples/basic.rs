use bgcss::{Element, Event};

fn main() {
  let path = std::path::PathBuf::from("./examples/basic.css");

  let parser = bgcss::parse(path);

  let elt = Element::named("hbox");

  loop {
    println!("Waiting parsing event");
    let event = parser.thread.recv();
    match event {
      Err(e) => {
        println!("Got error: {:?}", e);
        return;
      },
      Ok(Event::Error(e)) => {
        println!("Got error: {:?}", e);
      },
      Ok(Event::Parsed(rules)) => {
        rules.compute(&elt);
        println!("Event: Parsed");
      },
      Ok(event) => {
        println!("Event: {:?}", event);
      },
    }
  }
}
