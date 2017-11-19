extern crate simple_vm;

use simple_vm::Machine;
use std::env;

pub fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let code = args.join(" ");
    let mut machine = Machine::new_for_input(&code).unwrap();
    machine.run().unwrap();
}
