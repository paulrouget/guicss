# guicss ![License: MIT](https://img.shields.io/badge/license-MIT-blue) [![guicss on crates.io](https://img.shields.io/crates/v/guicss)](https://crates.io/crates/guicss) [![guicss on docs.rs](https://docs.rs/guicss/badge.svg)](https://docs.rs/guicss) [![Source Code Repository](https://img.shields.io/badge/Code-On%20github.com-blue)](https://github.com/paulrouget/guicss) [![guicss on deps.rs](https://deps.rs/repo/github/paulrouget/guicss/status.svg)](https://deps.rs/repo/github/paulrouget/guicss)

`GuiCss` is a CSS parser designed for Rust Desktop GUI.

**Warning:** Work In Progress.

The idea is to make it easier to theme any Rust GUI, iterate faster, or offer theme customisation to the end user.


## Features

 - The parser recompiles the CSS file as the user modifies CSS file;
 - Parsing runs in its dedicated thread;
 - The parser supports mediaQueries to write platform specific code (`os-version: macos|linux|windows`) and to match the OS theme (`prefers-color-scheme: light|dark`);
 - Computed properties are exported to a generic format that can be use with any toolkit. It also supports exporting to toolkit-specific style structures;
 - CSS variables are supported;


## CSS example


```css
@media (prefers-color-scheme: light) {
 :root {
   background-color: #EEE;
 }
 hbox {
   --mycolor: black;
   border: 2px solid blue;
 }
}

@media (prefers-color-scheme: dark) {
 hbox {
   --mycolor: white;
 }
}

hbox {
 color: var(--mycolor);
 background-color: red !important;
}

scrollarea::scrollbar {
 width: 12px;
}

@media (os-version: macos) {
 hbox {
   --toolbar-padding: 12px;
 }
}

```


## Example with [winit][__link0]


```rust
//! Basic example. Forwarding `guicss` events to Winit event loop,

use guicss::element::Element;
use guicss::parser::{parse_file, Event};
use winit::event::{Event as WinitEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopBuilder};

fn main() {
 let event_loop = EventLoopBuilder::with_user_event().build();
 let proxy = event_loop.create_proxy();

 let path = std::path::PathBuf::from("./examples/basic.css");
 let elt = Element::named("hbox").id("foo");

 parse_file(path, move |event| {
   if let Err(e) = proxy.send_event(event) {
     eprintln!("Sending user event failed: {e}");
   }
 });

 event_loop.run(move |event, _, control_flow| {
   *control_flow = ControlFlow::Wait;
   match event {
     WinitEvent::UserEvent(event) => {
       match event {
         Event::Error(e) => {
           eprintln!("Got error: {e}");
         },
         Event::Invalidated(new_rules) => {
           println!("Event: Parsed");
           let c = new_rules.compute(&elt);
           println!("Computed: {c:?}");
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

```


 [__link0]: https://lib.rs/winit
