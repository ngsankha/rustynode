use std::ptr;

use libc::c_char;

use js::{JSCLASS_RESERVED_SLOTS_MASK,JSCLASS_RESERVED_SLOTS_SHIFT,JSCLASS_GLOBAL_SLOT_COUNT,JSCLASS_IS_GLOBAL,JSPROP_ENUMERATE};
use js::jsapi::JS_GlobalObjectTraceHook;
use js::jsapi::{CallArgs,CompartmentOptions,OnNewGlobalHookOption,Rooted,Value};
use js::jsapi::{JS_DefineFunction,JS_Init,JS_InitStandardClasses,JS_NewGlobalObject,JS_EncodeStringToUTF8,JS_ReportError,JS_ReportPendingException,JS_CallFunctionName,CurrentGlobalOrNull,JS_SetReservedSlot,JS_GetReservedSlot,JS_NewStringCopyN,JS_GetClass,JS_FireOnNewGlobalObject,JS_SetPrototype};
use js::jsapi::{JSAutoCompartment,JSAutoRequest,JSContext,JSClass};
use js::jsapi::{JS_SetGCParameter,JSGCParamKey,JSGCMode};
use js::jsapi::{HandleValue,HandleValueArray,JSFunctionSpec,JSPropertySpec,JSNativeWrapper,JSTraceOp,JSObject,JSVersion,RootedObject,MutableHandleObject};
use js::jsval::{UndefinedValue,DoubleValue,StringValue,PrivateValue,ObjectValue};
use js::conversions::FromJSValConvertible;

use rustc_serialize::json;

use mio::{EventLoop,Handler};

use script::reflect::{Reflectable, PrototypeID, finalize, initialize_global};

#[derive(RustcDecodable, RustcEncodable)]
struct Timeout {
  id: u64,
  timeout: u64
}

pub struct EventLoopHandler;

pub struct Global {
  flag: u64,
  pub event_loop: EventLoop<EventLoopHandler>
}

static CLASS: JSClass = JSClass {
  name: b"Global\0" as *const u8 as *const c_char,
  flags: JSCLASS_IS_GLOBAL |
         (((JSCLASS_GLOBAL_SLOT_COUNT + 1) & JSCLASS_RESERVED_SLOTS_MASK) <<
          JSCLASS_RESERVED_SLOTS_SHIFT),
  addProperty: None,
  delProperty: None,
  getProperty: None,
  setProperty: None,
  enumerate: None,
  resolve: None,
  convert: None,
  finalize: Some(finalize::<Global>),
  call: None,
  hasInstance: None,
  construct: None,
  trace: Some(JS_GlobalObjectTraceHook),
  reserved: [0 as *mut _; 25],
};

static PROTOTYPE_CLASS: JSClass = JSClass {
  name: b"GlobalPrototype\0" as *const u8 as *const c_char,
  flags: 0,
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
  trace: None,
  reserved: [0 as *mut _; 25],
};

const METHODS: &'static [JSFunctionSpec] = &[
  JSFunctionSpec {
    name: b"_print\0" as *const u8 as *const c_char,
    call: JSNativeWrapper {op: Some(print), info: 0 as *const _},
    nargs: 1,
    flags: JSPROP_ENUMERATE as u16,
    selfHostedName: 0 as *const c_char
  },
  JSFunctionSpec {
    name: b"_send\0" as *const u8 as *const c_char,
    call: JSNativeWrapper {op: Some(send), info: 0 as *const _},
    nargs: 1,
    flags: JSPROP_ENUMERATE as u16,
    selfHostedName: 0 as *const c_char
  },
  JSFunctionSpec {
    name: 0 as *const c_char,
    call: JSNativeWrapper { op: None, info: 0 as *const _ },
    nargs: 0,
    flags: 0,
    selfHostedName: 0 as *const c_char
  }
];

impl Reflectable for Global {
  fn class() -> &'static JSClass {
    &CLASS
  }

  fn prototype_class() -> &'static JSClass {
    &PROTOTYPE_CLASS
  }

  fn attributes() -> Option<&'static [JSPropertySpec]> {
    None
  }

  fn methods() -> Option<&'static [JSFunctionSpec]> {
    Some(METHODS)
  }

  fn prototype_index() -> PrototypeID {
    PrototypeID::Global
  }
}

impl Global {
  fn print(&self, output: String) {
    println!("{}", output);
  }

  fn send(&self, event: String, message: String) {
    match event.as_str() {
      "timeout" => self.set_timeout(message),
      _ => ()
    };
  }

  fn set_timeout(&self, message: String) {
    let timeout_msg: Timeout = json::decode(message.as_str()).unwrap();
  }

  fn new() -> Global {
    let mut event_loop = EventLoop::new().unwrap();

    Global { flag: 0, event_loop: event_loop }
  }
}

impl Handler for EventLoopHandler {
  type Timeout = u64;
  type Message = (u64, String, String);

  fn timeout(&mut self, event_loop: &mut EventLoop<EventLoopHandler>, id: u64) {
    println!("{}", id);
    event_loop.shutdown();
  }
}

unsafe fn print_impl(cx: *mut JSContext, args: &CallArgs) -> Result<(), ()> {
  let global_obj = CurrentGlobalOrNull(cx);
  let global_root = Rooted::new(cx, ObjectValue(&*global_obj));
  let global = try!(Global::from_value(cx, global_root.handle()));
  let output = (0..args._base.argc_)
      .map(|i| String::from_jsval(cx, args.get(0), ()).unwrap())
      .collect::<Vec<String>>()
      .join(" ");
  (*global).print(output);

  args.rval().set(UndefinedValue());
  Ok(())
}

unsafe extern "C" fn print(cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
  let args = CallArgs::from_vp(vp, argc);
  print_impl(cx, &args).is_ok()
}

unsafe fn send_impl(cx: *mut JSContext, args: &CallArgs) -> Result<(), ()> {
  if args._base.argc_ != 2 {
    JS_ReportError(cx, b"_send() requires exactly 2 arguments\0".as_ptr() as *const c_char);
    return Err(());
  }

  let global_obj = CurrentGlobalOrNull(cx);
  let global_root = Rooted::new(cx, ObjectValue(&*global_obj));
  let global = try!(Global::from_value(cx, global_root.handle()));
  let event = try!(String::from_jsval(cx, args.get(0), ()));
  let message = try!(String::from_jsval(cx, args.get(1), ()));
  (*global).send(event, message);

  args.rval().set(UndefinedValue());
  Ok(())
}

unsafe extern "C" fn send(cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
  let args = CallArgs::from_vp(vp, argc);
  send_impl(cx, &args).is_ok()
}

pub fn create_global(cx: *mut JSContext, class: &'static JSClass, global: *mut Global, trace: JSTraceOp) -> *mut JSObject {
  unsafe {
    let mut options = CompartmentOptions::default();
    options.version_ = JSVersion::JSVERSION_ECMA_5;
    options.traceGlobal_ = trace;

    let obj = RootedObject::new(cx, JS_NewGlobalObject(cx, class, ptr::null_mut(), OnNewGlobalHookOption::DontFireOnNewGlobalHook, &options));
    assert!(!obj.ptr.is_null());
    let _ac = JSAutoCompartment::new(cx, obj.ptr);
    let global_boxed = unsafe { Box::from_raw(global) };
    global_boxed.init(obj.ptr);
    JS_InitStandardClasses(cx, obj.handle());
    initialize_global(obj.ptr);
    JS_FireOnNewGlobalObject(cx, obj.handle());
    obj.ptr
  }
}

pub unsafe fn create(cx: *mut JSContext, rval: MutableHandleObject) -> Box<Global> {
  let global = Box::into_raw(Box::new(Global::new()));
  rval.set(create_global(cx, &CLASS, global, None));
  let _ac = JSAutoCompartment::new(cx, rval.handle().get());
  let mut proto = RootedObject::new(cx, ptr::null_mut());
  Global::get_prototype_object(cx, rval.handle(), proto.handle_mut());
  assert!(JS_SetPrototype(cx, rval.handle(), proto.handle()));
  unsafe { Box::from_raw(global) }
}
