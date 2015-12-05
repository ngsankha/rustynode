use libc::c_void;

use js::error::throw_type_error;
use js::{JSCLASS_RESERVED_SLOTS_MASK,JSCLASS_RESERVED_SLOTS_SHIFT,JSCLASS_GLOBAL_SLOT_COUNT,JSCLASS_IS_GLOBAL};
use js::jsapi::JS_GlobalObjectTraceHook;
use js::jsapi::{CallArgs,CompartmentOptions,OnNewGlobalHookOption,Rooted,Value};
use js::jsapi::{JS_DefineFunction,JS_Init,JS_InitStandardClasses,JS_NewGlobalObject,JS_EncodeStringToUTF8,JS_ReportError,JS_ReportPendingException,JS_CallFunctionName,CurrentGlobalOrNull,JS_SetReservedSlot,JS_GetReservedSlot,JS_NewStringCopyN,JS_GetClass,JS_GetObjectPrototype,JS_NewObjectWithUniqueType};
use js::jsapi::{JSAutoCompartment,JSAutoRequest,JSContext,JSClass};
use js::jsapi::{JS_SetGCParameter,JSGCParamKey,JSGCMode,JSPropertySpec,JSFunctionSpec,JSFreeOp};
use js::jsapi::{HandleValue,HandleValueArray,JSObject,HandleObject,MutableHandleObject,RootedObject};
use js::jsval::{UndefinedValue,DoubleValue,StringValue,PrivateValue};
use js::rust::define_methods;
use js::rust::define_properties;
use js::rust::GCMethods;

const PRIVATE_OBJECT_SLOT: u32 = 0;

pub trait Reflectable: Sized {
  fn class() -> &'static JSClass;
  fn prototype_class() -> &'static JSClass;

  unsafe fn init(self: Box<Self>, object: *mut JSObject) {
    JS_SetReservedSlot(object, PRIVATE_OBJECT_SLOT, PrivateValue(Box::into_raw(self) as *const _));
  }

  unsafe fn is(object: *mut JSObject) -> bool {
    JS_GetClass(object) == Self::class()
  }

  unsafe fn from_reflector(object: *mut JSObject) -> *const Self {
    assert!(Self::is(object));
    let slot = JS_GetReservedSlot(object, PRIVATE_OBJECT_SLOT);
    slot.to_private() as *const _
  }

  unsafe fn from_value(cx: *mut JSContext, v: HandleValue) -> Result<*const Self, ()> {
    if !v.is_object() {
      throw_type_error(cx, "Value is not an object");
      return Err(());
    }

    let object = v.to_object();
    if !Self::is(object) {
      throw_type_error(cx, "Value is not an object");
      return Err(());
    }

    Ok(Self::from_reflector(object))
  }

  fn methods() -> Option<&'static [JSFunctionSpec]> {
    None
  }

  fn attributes() -> Option<&'static [JSPropertySpec]> {
    None
  }

  unsafe fn get_parent_proto(cx: *mut JSContext, global: HandleObject) -> *mut JSObject {
    let parent_proto = JS_GetObjectPrototype(cx, global);
    assert!(!parent_proto.is_null());
    parent_proto
  }

  unsafe fn create_interface_prototype_object(cx: *mut JSContext, global: HandleObject, rval: MutableHandleObject) {
    assert!(rval.get().is_null());

    let parent_proto = RootedObject::new(cx, Self::get_parent_proto(cx, global));

    rval.set(JS_NewObjectWithUniqueType(cx, Self::prototype_class(), parent_proto.handle()));
    assert!(!rval.get().is_null());

    if let Some(methods) = Self::methods() {
      define_methods(cx, rval.handle(), methods).unwrap();
    }

    if let Some(attributes) = Self::attributes() {
      define_properties(cx, rval.handle(), attributes).unwrap();
    }
  }

  fn prototype_index() -> PrototypeID;

  unsafe fn get_prototype_object(cx: *mut JSContext, global: HandleObject, rval: MutableHandleObject) {
    let prototypes = get_prototypes(global.get());
    let cache: *mut *mut JSObject = &mut (*prototypes)[Self::prototype_index() as usize];
    if !(*cache).is_null() {
      rval.set(*cache);
      return;
    }

    Self::create_interface_prototype_object(cx, global, rval);
    assert!(!rval.get().is_null());

    *cache = rval.get();
    if <*mut JSObject>::needs_post_barrier(*cache) {
      <*mut JSObject>::post_barrier(cache)
    }
  }
}

pub unsafe extern "C" fn finalize<T: Reflectable>(_fop: *mut JSFreeOp, object: *mut JSObject) {
  let this = T::from_reflector(object);
  let _ = Box::from_raw(this as *mut T);
}

unsafe fn get_prototypes(global: *mut JSObject) -> *mut [*mut JSObject; PrototypeID::Count as usize] {
  JS_GetReservedSlot(global, PRIVATE_PROTOTYPE_SLOT).to_private() as *mut ProtoOrIfaceArray
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u16)]
pub enum PrototypeID {
  Global = 0,
  Count = 1
}

/// An array of `*mut JSObject` of size `PrototypeID::Count`.
pub type ProtoOrIfaceArray = [*mut JSObject; PrototypeID::Count as usize];

pub const PRIVATE_PROTOTYPE_SLOT: u32 = JSCLASS_GLOBAL_SLOT_COUNT;


/// Construct and cache the ProtoOrIfaceArray for the given global.
pub unsafe fn initialize_global(global: *mut JSObject) {
  let proto_array: Box<ProtoOrIfaceArray> =
      Box::new([0 as *mut JSObject; 1]);
  let box_ = Box::into_raw(proto_array);
  JS_SetReservedSlot(global,
                     PRIVATE_PROTOTYPE_SLOT,
                     PrivateValue(box_ as *const c_void));
}