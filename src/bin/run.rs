#[macro_use] extern crate clap;
extern crate simple_vm;

use simple_vm::*;
use std::fs::File;
use std::io::prelude::*;

fn main() {

    let matches = clap_app!(simple_vm_bin =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: "A simple stack based vm.")
        (@arg file: +required "Input file of the program to run")
        (@arg args: +multiple "args to pass to the program")
    ).get_matches();

    match run(&matches) {
        Ok(response) => std::process::exit(response.exit_code),
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    }

}

fn run(matches: &clap::ArgMatches) -> Result<RunResult, String> {

    let program = {
        let mut file = {
            let file = File::open(&matches.value_of("file").unwrap());
            if let Err(e) = file {
                return Err(format!("Could not open file: {}", e));
            }
            file.unwrap()
        };
        let mut contents = String::new();
        if let Err(e) = file.read_to_string(&mut contents) {
            return Err(format!("Could not read file: {}", e));
        }
        let code = tokenize(&contents);
        if let Err(e) = code {
            return Err(format!("Could not tokenize file: {}", e));
        }
        code.unwrap()
    };

    let args = {
        let tokens = match matches.values_of("args") {
            None => Ok(vec![]),
            Some(values) => tokenize(&values.collect::<Vec<_>>().join(" "))
        };
        if let Err(e) = tokens {
            return Err(format!("Could not tokenize args: {}", e));
        }
        tokens.unwrap()
    };

    let mut machine = {
        let machine = Machine::<DefaultSideEffect>::new(program);
        if let Err(e) = machine {
            return Err(format!("Could not create Machine: {}", e));
        }
        machine.unwrap()
    };

    match machine.run(args) {
        Ok(re) => Ok(re),
        Err(e) => Err(format!("Machine execution failed: {}", e))
    }

}

