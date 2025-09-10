# Web support

## Prerequisites

This guide assumes you have already read the [Quickstart](quickstart.md).

## How to

Moly Kit has been designed with web support from day one. To run your app on the web,
you will need to do it like this:

```shell
cargo makepad wasm --bindgen run -p your_application_package
```

```admonish info
You will need to have the `cargo makepad` CLI installed. Check Makepad's
documentation for more information.
```

```admonish warning
By default, Makepad uses its own glue code to work in a web browser and doesn't
work with `wasm-bindgen` out of the box.

The `--bindgen` argument we passed to `cargo makepad` earlier is very important
as it enables `wasm-bindgen` interoperability in Makepad.

But with that argument enabled, if we don't use `wasm-bindgen` in our app, we may
see `wasm-bindgen` related errors on the browser console related to missing values,
which are solved by ensuring we use `wasm-bindgen` somewhere in our app.
```

