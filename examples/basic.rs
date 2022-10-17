use bgcss::{Event, Element};

fn main() {

  let path = std::path::PathBuf::from("./examples/basic.css");

  let parser = bgcss::parse(path);

  let root = Element::named("foo");
  // let hbox = Element::named("hbox");
  // let vbox = Element::named("vbox");
  // let n11 = Element::named("node").id("n11");
  // let n12 = Element::named("node").id("n12");
  // let n13 = Element::unamed();
  // let scrollbar = Element::scrollbar();

  loop {
    println!("Waiting parsing event");
    let event = parser.thread.recv();
    match event {
      Err(e) => {
        println!("Got error: {:?}", e);
        return;
      },
      Ok(Event::Parsed(rules, errors)) => {

        for error in errors {
          println!("css error: {}", error);
        }

        rules.compute(&root);

        println!("Event: Parsed");
      },
      Ok(event) => {
        println!("Event: {:?}", event);
      },
    }
  }
}
