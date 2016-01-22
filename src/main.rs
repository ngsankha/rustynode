
#![feature(convert)]

extern crate js;
extern crate libc;
extern crate rustc_serialize;
extern crate mio;

mod script;

fn main() {
  script::run_script();
}