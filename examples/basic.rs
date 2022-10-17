use stylesheet::Element;

fn main() {

  let path = std::path::PathBuf::from("./examples/basic.css");
  let sheet = stylesheet::parse(path);

  let root = Element::named("foo");
  let hbox = Element::named("hbox");
  let vbox = Element::named("vbox");
  let n11 = Element::named("node").id("n11");
  let n12 = Element::named("node").id("n12");
  let n13 = Element::unamed();
  let scrollbar = Element::scrollbar();

  sheet.compute(&root);

  loop {
    println!("Waiting parsing event");
    let event = sheet.thread.recv();
    match event {
      Err(e) => {
        println!("Got error: {:?}", e);
        return;
      },
      Ok(stylesheet::Event::Parsed) => {
        println!("Event: Parsed");
      },
      Ok(event) => {
        println!("Event: {:?}", event);
      },
    }
  }
}
