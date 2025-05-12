# Web support

## Prerequisites

This guide assumes you have already read the [Quickstart](quickstart.md).

## How to

Moly Kit has been designed with web support from day one. If you are targeting
only the web, you simply need to do the following:

Add `wasm_bindgen` to your app.

```shell
cargo add wasm_bindgen
```

Then, ensure you use its prelude somewhere. Let's just place it in the main file.

```rust
use wasm_bindgen::prelude::*;

fn main() {
    your_application_lib::app::app_main()
}
```

Finally, to run your app, you will need to do it like this:

```shell
cargo makepad wasm --bindgen run -p your_application_package
```

> **Note:** You will need to have the `cargo makepad` CLI installed. Check Makepad's
> documentation for more information.

## A warning about `wasm-bindgen` support in Makepad

By default, Makepad uses its own glue code to work in a web browser and doesn't
work with `wasm-bindgen` out of the box.

The `--bindgen` argument we passed to `cargo makepad` earlier is very important
as it enables `wasm-bindgen` interoperability in Makepad. 

## Targeting both web and native (non-web)

If you want to be able to build your app for both web and non-web platforms,
you can adapt your main file to the following conditionally compiled code:

```rust
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
fn main() {
    your_application_lib::app::app_main()
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    your_application_lib::app::app_main()
}
```

Please notice how the `main` function targeting native platforms needs to run in
the context of a Tokio runtime.

However, on the web, we can't use Tokio, as the browser has its own way of driving futures.
