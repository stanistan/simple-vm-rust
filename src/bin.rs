extern crate simple_vm;

use simple_vm::{Machine, tokenize};
use std::env;
use std::io::prelude::*;
use std::fs::File;

pub fn main() {

    let file_path: String = env::args().skip(1).take(1).collect();
    if file_path.is_empty() {
        panic!("Expected a file path");
    }

    let mut f = File::open(&file_path).expect("File does not exist");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("Reading the file failed");

    let code = tokenize(&contents).expect("Could not tokenize file contents");
    let mut machine = Machine::new(code).expect("Could not create machine.");
    let stats = machine.run().expect("Code execution failed");
    println!("{:#?}", stats);
}
