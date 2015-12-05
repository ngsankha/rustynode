mod global;
mod reflect;

use std::sync::{Once, ONCE_INIT};
use std::ptr;

use js::jsapi::{JS_Init, JSAutoRequest, Rooted};
use js::rust::Runtime;

static INIT: Once = ONCE_INIT;

pub fn run_script() -> Result<(), ()> {
  INIT.call_once(|| {
    unsafe { assert!(JS_Init()) };
  });

  let runtime = Runtime::new();
  let _ar = JSAutoRequest::new(runtime.cx());
  let mut global = Rooted::new(runtime.cx(), ptr::null_mut());
  unsafe { global::create(runtime.cx(), global.handle_mut()) };
  assert!(!global.ptr.is_null());

  try!(runtime.evaluate_script(global.handle(), "_print(\"Hello\");".to_string(), "abc.js".to_string(), 0));
  Ok(())
}
