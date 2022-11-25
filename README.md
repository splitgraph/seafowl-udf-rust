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

This repository includes the `make_create_function_sql.js` helper script to create the
`CREATE FUNCTION ...` expression used to define the Seafowl UDF.

To load the example `add_i64()` function into Seafowl, run the following:

```bash
node make_create_function_sql.js add_i64 add_i64 target/wasm32-wasi/release/seafowl_udf_rust.wasm > create_udf.sql
seafowl/examples/clients/node/seafowl-client.js -f create_udf.sql
```

Invoking the newly created UDF:

```bash
seafowl/examples/clients/node/seafowl-client.js 'SELECT add_i64(1,2)'
```

The input types and return type of the UDF is hardcoded into `make_create_function_sql.js`,
override these for UDF signatures other than `(i64, i64) -> i64`.

# Running unit tests

```bash
cargo test
```
