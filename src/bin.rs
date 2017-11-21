#![feature(libc)]
extern crate simple_vm;

use simple_vm::{Machine, tokenize};
use std::env;
use std::io::prelude::*;
use std::fs::File;

#[cfg(feature="mem-usage")]
extern crate libc;

#[cfg(feature="mem-usage")]
extern {
    fn je_stats_print(
        write_cb: extern fn (*const libc::c_void, *const libc::c_char),
        cbopaque: *const libc::c_void, opts: *const libc::c_char
    );
}

#[cfg(feature="mem-usage")]
extern fn write_cb (_: *const libc::c_void, message: *const libc::c_char) {
    print!("{}", String::from_utf8_lossy(unsafe {
        std::ffi::CStr::from_ptr(message as *const i8).to_bytes()
    }));
}

pub fn main() {
    let file_path: String = env::args().skip(1).take(1).collect();
    if file_path.is_empty() {
        panic!("Expected a file path");
    }

    let script_args: Vec<String> = env::args().skip(2).collect();
    let args = tokenize(&script_args.join(" ")).expect("could not parse args");

    let mut f = File::open(&file_path).expect("File does not exist");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("Reading the file failed");

    let code = tokenize(&contents).expect("Could not tokenize file contents");
    let mut machine = Machine::new(code).expect("Could not create machine.");
    let stats = machine.run(args).expect("Code execution failed");
    if let Some(stats) = stats {
        println!("{:#?}", stats);
    }

    #[cfg(feature="mem-usage")]
    unsafe {
        je_stats_print(write_cb, std::ptr::null(), std::ptr::null())
    };
}
