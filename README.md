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

```
cargo test
```

#### Examples

Run the fib program for the 5th fibonacci number (debug).

```
cargo run -- examples/fib 5
```

Run this in release mode.

```
cargo run --release -- examples/fib 5
```

---

Run this with `stats`, `mem-usage`, and `debug` outputs, assuming you have [`rustup`](https://www.rustup.rs):

```
rustup run nightly cargo run --release --features=stats,mem-usage,debug -- examples/fib 5
```

---

Benchmarking

```
rustup run nightly cargo bench --features=bench
```
