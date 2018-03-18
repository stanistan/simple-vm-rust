#![feature(proc_macro)]
#![allow(non_camel_case_types)]

extern crate simple_vm;
extern crate wasm_bindgen;

use simple_vm::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    type console;
    #[wasm_bindgen(static = console)]
    fn log(s: &str);
    #[wasm_bindgen(static = console)]
    fn warn(s: &str);
}

macro_rules! try_js {
    ($e: expr) => {
        match $e {
            Ok(ok) => {
                console::log(&format!("Success: {} {:?}", stringify!($e), &ok));
                ok
            },
            Err(e) => {
                console::warn(&format!("Error: {} {:?}", stringify!($e), e));
                return JsValue::null();
            }
        }
    }
}

#[wasm_bindgen]
pub fn run(code: &str, args: &str) -> JsValue {
    let code = try_js!(tokenize(code));
    let args = try_js!(tokenize(args));
    let mut machine = try_js!(Machine::new(code));
    let _stats = try_js!(machine.run(args));
    JsValue::from_str(&format!("stack: {:#?}", machine.stack()))
}
