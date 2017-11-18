extern crate simple_vm;

use simple_vm::Machine;
use std::env;

pub fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut machine = Machine::new(args);
    machine.run().unwrap();
}
