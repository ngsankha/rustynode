extern crate js;
extern crate libc;
extern crate rustc_serialize;
extern crate mio;

mod script;

use std::ffi::CStr;
use std::ptr;
use std::str;
use std::io::prelude::*;
use std::fs::File;
use std::thread;
use std::env;
use std::sync::{Once, ONCE_INIT};

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

static INIT: Once = ONCE_INIT;

fn main() {
  script::run_script();
}