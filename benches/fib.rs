#![feature(test)]

extern crate simple_vm;
extern crate test;

use simple_vm::*;
use test::Bencher;

/// Path to fib example that doesn't print when
/// it's finished.
static FIB_CODE: &'static str = include_str!("../examples/fib_no_print");

#[bench]
fn bench_tokenize_fib(b: &mut Bencher) {
    b.iter(move || {
        tokenize(FIB_CODE).unwrap();
    });
}

#[bench]
fn bench_create_machine(b: &mut Bencher) {
    let code = tokenize(FIB_CODE).unwrap();
    b.iter(move || {
        Machine::<DefaultSideEffect>::new(code.clone()).unwrap();
    });
}

macro_rules! bench_fib_arg {
    ($ident:ident, $arg:expr) => {
        #[bench]
        fn $ident(b: &mut Bencher) {
            let code = tokenize(FIB_CODE).unwrap();
            let args = tokenize($arg).unwrap();
            let mut machine = Machine::<DefaultSideEffect>::new(code).unwrap();
            b.iter(move || {
                machine.reset();
                machine.run(args.clone()).unwrap();
            });
        }
    };
}

bench_fib_arg!(bench_fib_1, "1");
bench_fib_arg!(bench_fib_5, "5");
bench_fib_arg!(bench_fib_10, "10");
