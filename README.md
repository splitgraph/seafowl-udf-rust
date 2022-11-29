# seafowl-udf-rust
Example WASM User Defined Function (UDF) for [Seafowl](https://seafowl.io/) in [Rust](https://www.rust-lang.org/).

# Dependencies

First [install Rust and Cargo](https://www.rust-lang.org/tools/install), then install [cargo-wasi](https://github.com/bytecodealliance/cargo-wasi) by running:

```bash
cargo install cargo-wasi
```


# Building the WASM module

```bash
cargo wasi build --release
```

# Adding your own functions

Just like the [`add_i64`](src/lib.rs#L12-L21) example function, for Seafowl to successfully invoke your UDF, it must:
1. Use the `#[no_mangle]` attribute, otherwise the exported function will be renamed and Seafowl will not be able to load it.
1. Accept and return pointers to [msgpack](https://msgpack.org/index.html)-serialized buffers. The `wrap_udf` function takes care of (de)serialization, as shown by the demo function.

Messages sent to `stderr` via `eprintln!()` will be visible in the Seafowl console.

A WASM module may include multiple user-defined functions, but each one must be loaded with a separate `CREATE FUNCTION ...` statement.

# Loading the WASM module into Seafowl as a UDF

This repository includes the `create_udf.sh` shell script which creates the Seafowl function wrapping the Rust WASM logic.
Be sure to update the parameters within this file to match your own UDFs' name and signature.

To use the script as-is, start a local Seafowl instance with HTTP write access enabled, eg:

```bash
SEAFOWL__FRONTEND__HTTP__WRITE_ACCESS=any ./target/release/seafowl
```

To load the example `add_i64()` function into a locally running Seafowl, just run:

```bash
./create_udf.sh 
```

Invoking the newly created UDF:

```bash
./query_udf.sh
```

Edit `query_udf.sh` to change the function's arguments or run more complex queries.

# Running unit tests

```bash
cargo test
```
