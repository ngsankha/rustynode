extern crate js;
extern crate libc;
extern crate rustc_serialize;
extern crate mio;

use std::ffi::CStr;
use std::ptr;
use std::str;
use std::io::prelude::*;
use std::fs::File;
use std::thread;

use js::{JSCLASS_RESERVED_SLOTS_MASK,JSCLASS_RESERVED_SLOTS_SHIFT,JSCLASS_GLOBAL_SLOT_COUNT,JSCLASS_IS_GLOBAL};
use js::jsapi::JS_GlobalObjectTraceHook;
use js::jsapi::{CallArgs,CompartmentOptions,OnNewGlobalHookOption,Rooted,Value};
use js::jsapi::{JS_DefineFunction,JS_Init,JS_InitStandardClasses,JS_NewGlobalObject,JS_EncodeStringToUTF8,JS_ReportError,JS_ReportPendingException,JS_CallFunctionName,CurrentGlobalOrNull,JS_SetReservedSlot,JS_GetReservedSlot,JS_NewStringCopyN};
use js::jsapi::{JSAutoCompartment,JSAutoRequest,JSContext,JSClass};
use js::jsapi::{JS_SetGCParameter,JSGCParamKey,JSGCMode};
use js::jsapi::{HandleValue,HandleValueArray};
use js::jsval::{UndefinedValue,DoubleValue,StringValue,PrivateValue};
use js::rust::Runtime;

use rustc_serialize::json;

use mio::{EventLoop,Handler};

static CLASS: &'static JSClass = &JSClass {
  name: b"Global\0" as *const u8 as *const libc::c_char,
  flags: JSCLASS_IS_GLOBAL | ((JSCLASS_GLOBAL_SLOT_COUNT & JSCLASS_RESERVED_SLOTS_MASK) << JSCLASS_RESERVED_SLOTS_SHIFT),
  addProperty: None,
  delProperty: None,
  getProperty: None,
  setProperty: None,
  enumerate: None,
  resolve: None,
  convert: None,
  finalize: None,
  call: None,
  hasInstance: None,
  construct: None,
  trace: Some(JS_GlobalObjectTraceHook),
  reserved: [0 as *mut _; 25]
};

#[derive(RustcDecodable, RustcEncodable)]
struct Timeout {
  id: u64,
  timeout: u64
}

#[derive(RustcDecodable, RustcEncodable)]
struct FileRead {
  id: u64,
  filename: String
}

struct EventLoopHandler {
  rt: Runtime
}

impl Handler for EventLoopHandler {
  type Timeout = u64;
  type Message = (u64, String, String);

  fn notify(&mut self, event_loop: &mut EventLoop<EventLoopHandler>, message: (u64, String, String)) {
    let cx = self.rt.cx();
    let _ar = JSAutoRequest::new(cx);
    unsafe {
      let global = CurrentGlobalOrNull(cx);
      let mut rval = Rooted::new(cx, UndefinedValue());
      assert!(!global.is_null());
      let global_root = Rooted::new(cx, global);
      let event_jsstr = JS_NewStringCopyN(cx, message.1.as_ptr() as *const libc::c_char, message.1.len());
      let data_jsstr = JS_NewStringCopyN(cx, message.2.as_ptr() as *const libc::c_char, message.2.len());
      let elems = [StringValue(&*event_jsstr), DoubleValue(message.0 as f64), StringValue(&*data_jsstr)];
      let args = HandleValueArray{ length_: 3, elements_: &elems as *const Value };
      JS_CallFunctionName(cx, global_root.handle(), b"_recv\0".as_ptr() as *const libc::c_char, &args, rval.handle_mut());
    }
  }

  fn timeout(&mut self, event_loop: &mut EventLoop<EventLoopHandler>, id: u64) {
    let cx = self.rt.cx();
    let _ar = JSAutoRequest::new(cx);
    unsafe {
      let global = CurrentGlobalOrNull(cx);
      let mut rval = Rooted::new(cx, UndefinedValue());
      assert!(!global.is_null());
      let global_root = Rooted::new(cx, global);
      let event_jsstr = JS_NewStringCopyN(cx, b"timeout\0".as_ptr() as *const libc::c_char, 7);
      let elems = [StringValue(&*event_jsstr), DoubleValue(id as f64)];
      let args = HandleValueArray{ length_: 2, elements_: &elems as *const Value };
      JS_CallFunctionName(cx, global_root.handle(), b"_recv\0".as_ptr() as *const libc::c_char, &args, rval.handle_mut());
    }
    //event_loop.shutdown();
  }
}

fn set_timeout(cx: *mut JSContext, message: &str) {
  let timeout_msg: Timeout = json::decode(message).unwrap();
  let _ar = JSAutoRequest::new(cx);
  unsafe {
    let global = CurrentGlobalOrNull(cx);
    assert!(!global.is_null());
    let value = JS_GetReservedSlot(global, 0);
    assert!(!value.is_undefined());
    let event_loop = value.to_private() as *mut EventLoop<EventLoopHandler>;
    let _ = (*event_loop).timeout_ms(timeout_msg.id, timeout_msg.timeout);
  };
}

fn read_file(cx: *mut JSContext, message: &str) {
  let readfile_msg: FileRead = json::decode(message).unwrap();
  let _ar = JSAutoRequest::new(cx);
  let sender = unsafe {
    let global = CurrentGlobalOrNull(cx);
    assert!(!global.is_null());
    let value = JS_GetReservedSlot(global, 0);
    assert!(!value.is_undefined());
    let event_loop = value.to_private() as *mut EventLoop<EventLoopHandler>;
    (*event_loop).channel()
  };

  thread::spawn(move || {
    let mut f = File::open(readfile_msg.filename).unwrap();
    let mut data = String::new();
    f.read_to_string(&mut data);
    let _ = sender.send((readfile_msg.id, "readFile".to_string(), data));
  });
}

fn callback(cx: *mut JSContext, event: &str, message: &str) {
  match event {
    "timeout" => set_timeout(cx, message),
    "readFile" => read_file(cx, message),
    _ => ()
  };
}

fn main() {
  unsafe {
    JS_Init();
  }
  let runtime = Runtime::new();
  let cx = runtime.cx();

  let h_option = OnNewGlobalHookOption::FireOnNewGlobalHook;
  let c_option = CompartmentOptions::default();
  let _ar = JSAutoRequest::new(cx);
  let global = unsafe { JS_NewGlobalObject(cx, CLASS, ptr::null_mut(), h_option, &c_option) };
  let global_root = Rooted::new(cx, global);
  let global = global_root.handle();
  let _ac = JSAutoCompartment::new(cx, global.get());
  unsafe {
    JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_MODE, JSGCMode::JSGC_MODE_INCREMENTAL as u32);
    JS_InitStandardClasses(cx, global);
    let send_fn = JS_DefineFunction(cx, global, b"_send\0".as_ptr() as *const libc::c_char,
                                    Some(send), 1, 0);
    assert!(!send_fn.is_null());
    let print_fn = JS_DefineFunction(cx, global, b"_print\0".as_ptr() as *const libc::c_char,
                                     Some(print), 1, 0);
    assert!(!print_fn.is_null());
  }

  let event_loop = EventLoop::new().unwrap();
  let mut boxed_event_loop = Box::new(event_loop);
  let mut handler = EventLoopHandler { rt: runtime };
  let box_ptr = Box::into_raw(boxed_event_loop);

  unsafe {
    JS_SetReservedSlot(global.get(), 0,
                       PrivateValue(box_ptr as *const libc::c_void));
    boxed_event_loop = Box::from_raw(box_ptr);
  }

  let mut file = match File::open("src/bootstrap.js") {
    Err(_) => panic!("Error opening file"),
    Ok(file) => file
  };
  let mut source = String::new();
  if let Err(_) = file.read_to_string(&mut source) {
    panic!("Error reading file");
  };
  match handler.rt.evaluate_script(global, source, "bootstrap.js".to_string(), 1) {
    Err(_) => unsafe { JS_ReportPendingException(cx); panic!("Error executing JS") },
    _ => ()
  };
  let _ = &boxed_event_loop.run(&mut handler);
}

fn jsval_to_str(cx: *mut JSContext, val: HandleValue) -> String {
  let js = unsafe { js::rust::ToString(cx, val) };
  let rooted_val = Rooted::new(cx, js);
  let string = unsafe {
    let tmp = JS_EncodeStringToUTF8(cx, rooted_val.handle());
    CStr::from_ptr(tmp)
  };
  str::from_utf8(string.to_bytes()).unwrap().to_string()
}

unsafe extern "C" fn send(cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
  let args = CallArgs::from_vp(vp, argc);

  if args._base.argc_ != 2 {
    JS_ReportError(cx, b"_send() requires exactly 2 arguments\0".as_ptr() as *const libc::c_char);
    return false;
  }

  let event = jsval_to_str(cx, args.get(0));
  let message = jsval_to_str(cx, args.get(1));
  callback(cx, &event, &message);

  args.rval().set(UndefinedValue());
  return true;
}

unsafe extern "C" fn print(cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
    let args = CallArgs::from_vp(vp, argc);

    let output = (0..args._base.argc_)
        .map(|i| fmt_js_value(cx, args.get(i)))
        .collect::<Vec<String>>()
        .join(" ");
    println!("{}", output);

    args.rval().set(UndefinedValue());
    return true;
}

fn fmt_js_value(cx: *mut JSContext, val: HandleValue) -> String {
    let js = unsafe { js::rust::ToString(cx, val) };
    let message_root = Rooted::new(cx, js);
    let message = unsafe { JS_EncodeStringToUTF8(cx, message_root.handle()) };
    let message = unsafe { CStr::from_ptr(message) };
    String::from(str::from_utf8(message.to_bytes()).unwrap())
}
