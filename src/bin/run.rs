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
        (@arg dump_ast: --ast "Print the Machine's code before running")
        (@arg no_run: --no_run "Don't execute the program")
        (@arg step: --step "Step through the program one operation at a time.")
        (@arg args: +multiple "args to pass to the program")
    ).get_matches();

    match run(&matches) {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    }

}

/// Like `try!` but will transform all `Err(e)` into `Err(String)`
macro_rules! attempt {
    ($msg:expr => $e:expr) => {
        match $e {
            Err(e) => return Err(format!("Error {}... {}", $msg, e)),
            Ok(v) => v
        }
    }
}

/// Attempts to actually run the program
fn run(matches: &clap::ArgMatches) -> Result<i32, String> {

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


    let mut machine = attempt!("creating machine" => Machine::<DefaultSideEffect>::new(program));

    if matches.is_present("step") {
        machine.enable_step();
    }

    if matches.is_present("dump_ast") {
        println!("{:?}", machine.code);
    }

    let mut exit_code = 0;
    if !matches.is_present("no_run") {
        exit_code = attempt!("running machine" => machine.run(args)).exit_code;
    }

    Ok(exit_code)
}

