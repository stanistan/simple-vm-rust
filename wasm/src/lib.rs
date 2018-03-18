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
                return Vec::new();
            }
        }
    }
}

fn to_js_value(stack_value: &StackValue) -> JsValue {
    use StackValue::*;
    match *stack_value {
        Bool(b) => b.into(),
        Num(n) => (n as u32).into(),
        _ => (&format!("{}", stack_value)).into()
    }
}

#[wasm_bindgen]
pub fn run(code: &str, args: &str) -> Vec<JsValue> {
    let code = try_js!(tokenize(code));
    let args = try_js!(tokenize(args));
    let mut machine = try_js!(Machine::new(code));
    let _stats = try_js!(machine.run(args));
    let ret: Vec<JsValue> = machine.stack()
        .iter()
        .map(|v| to_js_value(v))
        .collect();
    ret
}
