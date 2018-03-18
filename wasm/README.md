All of the wasm stuff is based off of [wasm-bindgen](https://github.com/alexcrichton/wasm-bindgen).

Keeping the node server running (watching for changes) doesn't _super_ work out. I should
probably just use the `build.sh` scripts that are included in the demos from the repo
above.

### Running everything...

This should only be run once (or, when dependencies change).

```sh
npm install
```

Make sure `rustup` is on the _nightly_ toolchain from the root (`simple_vm`) project.

```sh
# this toolchain should already be installed
# given the wasm-bindgen docs
cargo +nightly build --release --target wasm32-unknown-unkown
wasm-bindgen target/wasm32-unknown-unknown/release/simple_vm_wasm.wasm --out-dir .
```

And run the JS

```sh
npm run serve
```
