#[macro_use] extern crate clap;
extern crate simple_vm;

use simple_vm::*;
use std::fs::File;
use std::io::Read;

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

macro_rules! attempt {
    ($msg:expr => $e:expr) => {
        match $e {
            Err(e) => return Err(format!("Error {}... {}", $msg, e)),
            Ok(v) => v
        }
    }
}

type DefaultMachine = Machine<DefaultSideEffect>;

fn run(matches: &clap::ArgMatches) -> Result<RunResult, String> {

    let program = {
        let file_name = matches.value_of("file").unwrap();
        let mut file = attempt!("opening file" => File::open(&file_name));
        let mut contents = String::new();
        attempt!("reading file" => file.read_to_string(&mut contents));
        attempt!("tokenizing file" => tokenize(&contents))
    };

    let args = {
        let args: Vec<_> = matches.values_of("args")
            .unwrap_or_default()
            .collect();
        attempt!("tokenizng args" => tokenize(&args.join(" ")))
    };

    let mut machine = attempt!("creating machine" => DefaultMachine::new(program));
    Ok(attempt!("running machine" => machine.run(args)))
}

