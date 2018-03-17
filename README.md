# simple vm rust

[![Build Status](https://travis-ci.org/stanistan/simple-vm-rust.svg?branch=master)](https://travis-ci.org/stanistan/simple-vm-rust)

This is an implementation of a super simple stack based VM in Rust, as a learning
exercise for myself. Absolutely do not use this for anything real or production worthy.

The language and APIs will change over time.

A lot of this is based on https://csl.name/post/vm, but while that is written in Python,
this is in not :) So there are obviously some more constraints in how this is run.

## The Language

Right now this is a super simple stack based language that supports:

1. integers (no floats yet)
2. strings
3. labels
4. jmp / call / return

It does allocation for every time something is moved on the stack... instead of using
references or reference counting, values will be heap allocated every time they're pushed,
and cloned from the code when it's being executed, this is currently pretty inefficient and
I'm hoping to make it better.

## Running

Assuming you have [`rustup`](https://www.rustup.rs).

#### tests

```sh
cargo test
```

#### Examples

Run the fib program for the 5th fibonacci number (debug).

```sh
cargo run -- examples/fib 5
```

Run this in release mode.

```sh
cargo run --release -- examples/fib 5
```

---

Run this with `stats`, `mem-usage`, and `debug` outputs, assuming you have [`rustup`](https://www.rustup.rs):

```sh
rustup run nightly cargo run --release --features=stats,mem-usage,debug -- examples/fib 5
```

---

Benchmarking

```
cd bench
cargo +nightly bench
```

#### Getting Flamegraphs

(Using [this](https://github.com/brendangregg/FlameGraph))

The `Cargo.toml` has `debug=true` in the `[profile.release]` section so that symbols
don't get mangled and we can do perf tracing.

```sh
cargo build --release
# trace for doing running `examples/fib 10`
sudo dtrace \
    -c 'target/release/simple_vm examples/fib 10' \
    -o stack.out \
    -n 'profile-10001 { @[ustack()] = count() }'
stackcollapse.pl stack.out | flamegraph.pl > graph.svg
open graph.svg
```

If trying to get a trace for a single benchmark:

```sh
# deleting this to simplify finding the right binary.
rm -rf target/release/
cargo clean
rustup run nightly cargo bench --no-run
```

One of the files matching `target/release/simple_vm-*` will end up being the correct executable.

```sh
sudo dtrace \
    -c 'target/release/simple_vm-$HASH --bench $TEST' \
    -o stack.out \
    -n 'profile-10001 { @[ustack()] = count() }'

stackcollapse.pl stack.out | flamegraph.pl > graph.svg
open graph.svg
```

![bench.svg](./bench.svg)
