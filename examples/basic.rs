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
      Ok(e) => {
        println!("Got parsing event: {:?}", e);
      },
    }
  }
}
