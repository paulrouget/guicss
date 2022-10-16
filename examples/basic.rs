use stylesheet::{Arena, Element, NodeId, StyleSheet};

fn main() {

  let path = std::path::PathBuf::from("./examples/basic.css");
  let sheet = stylesheet::parse(path);
  let arena = &mut stylesheet::Arena::new();
  let root = build_tree(arena);

  compute(&sheet, arena, root);

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
        compute(&sheet, arena, root);
      },
      Ok(event) => {
        println!("Event: {:?}", event);
      },
    }
  }
}

fn compute(sheet: &StyleSheet, arena: &Arena, top: NodeId) {
  for id in top.descendants(arena) {
    let props = sheet.compute(arena, id);
    println!("Found node {:?}", id);
    println!("Computation result: {:?}", props);
  }
}

fn build_tree(arena: &mut Arena) -> NodeId {
  // <foo>
  //   <hbox>
  //     <node id="n11"></node>
  //     <node id="n12"></node>
  //     <???></???>
  //   </hbox>
  //   <vbox></vbox>
  // </foo>

  let root = arena.new_node(Element::new().name("foo"));

  let hbox = arena.new_node(Element::new().name("hbox"));
  let vbox = arena.new_node(Element::new().name("vbox"));

  root.append(hbox, arena);
  root.append(vbox, arena);

  let n11 = arena.new_node(Element::new().name("node").id("n11"));
  let n12 = arena.new_node(Element::new().name("node").id("n12"));
  let n13 = arena.new_node(Element::new());

  hbox.append(n11, arena);
  hbox.append(n12, arena);
  hbox.append(n13, arena);

  root
}
