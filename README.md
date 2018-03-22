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

#### Benchmarking

```
cd bench
cargo +nightly bench
```

Use [`cargo benchcmp`](https://github.com/BurntSushi/cargo-benchcmp) for bench comparisons.

#### Getting Flamegraphs

The idea here is to run a profiler inside a docker container to actually get some flamegraphs
out, since running them on MacOS is [pretty unweildy](http://carol-nichols.com/2015/12/09/rust-profiling-on-osx-cpu-time/),
and just straight up might not work at all, but it is [usable elsewhere](https://blog.anp.lol/rust/2016/07/24/profiling-rust-perf-flamegraph/).

We are also using [this](https://github.com/brendangregg/FlameGraph), so that should be somewhere
and available to you/me.

```sh
docker build -t simple-vm-perf . -f perf/Dockerfile
```

The Dockerfile will install `perf` so it can be un inside of the container and builds
out the `simple_vm` project with the `bench/` sub-project.

The first build will be a little expensive since it's going to be downloading all
of the dependencies, etc, but the Dockerfile is structured in a way to minimize
re-downloading/compiling everything (unless you're adding or removing dependencies).

- [ ] Use multistage builds

Ok, so once we have that built out we can run it!

```sh
docker run --privileged -it -v "`pwd`/prof":/prof simple-vm-perf /prof/run.sh fib_10
cat prof/data/perf.script | stackcollapse-perf.pl | flamegraph.pl > bench.svg
```

(That assumes that Flamegraph things are in your `$PATH`)

![bench.svg](./bench.svg)
