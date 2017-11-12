extern crate simple_vm;

pub fn main() {
    use simple_vm::run;
    use std::env;
    let args: Vec<String> = env::args().skip(1).collect();
    run(args);
}
