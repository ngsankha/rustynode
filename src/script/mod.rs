mod global;
mod reflect;

use std::sync::{Once, ONCE_INIT};
use std::ptr;
use std::path::Path;
use std::fs::File;
use std::io::Read;

use js::jsapi::{JS_Init, JSAutoRequest, Rooted};
use js::rust::Runtime;

static INIT: Once = ONCE_INIT;

fn load_script(path: &Path) -> String {
  // TODO: Make this return a Result<String, Error>
  let mut file = File::open(path).unwrap();
  let mut buffer = vec![];
  file.read_to_end(&mut buffer).unwrap();
  let script = String::from_utf8(buffer).unwrap();
  script
}

pub fn run_script() -> Result<(), ()> {
  INIT.call_once(|| {
    unsafe { assert!(JS_Init()) };
  });

  let runtime = Runtime::new();
  let _ar = JSAutoRequest::new(runtime.cx());
  let mut global = Rooted::new(runtime.cx(), ptr::null_mut());
  let mut global_obj = unsafe { global::create(runtime.cx(), global.handle_mut()) };
  assert!(!global.ptr.is_null());
  let bootstrap_script = load_script(Path::new("lib/bootstrap.js"));
  let user_script = load_script(Path::new("examples/readfile.js"));
  try!(runtime.evaluate_script(global.handle(), bootstrap_script, "bootstrap.js".to_string(), 0));

  try!(runtime.evaluate_script(global.handle(), user_script, "test.js".to_string(), 0));
  let mut handler = global::EventLoopHandler { runtime: runtime, js_global: global.ptr };
  global_obj.event_loop.run(&mut handler);
  Ok(())
}
