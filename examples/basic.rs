fn main() {
  let path = std::path::PathBuf::from("./examples/basic.css");
  let sheet = stylesheet::parse(path);

  loop {
    println!("Waiting parsing event");
    let event = sheet.thread.recv();
    match event {
      Err(e) => {
        println!("Got error: {:?}", e);
        return;
      },
      Ok(stylesheet::Event::Parsed(errors)) => {
        println!("Event: Parsed");
        for e in errors {
          println!("CSS Error: {:?}", e);
        }
      },
      Ok(event) => {
          println!("Event: {:?}", event);
      }
    }
  }
}
