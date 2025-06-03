#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
fn main() {
    // Initialize the logger
    env_logger::init();
    moly::app::app_main()
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Initialize the logger
    env_logger::init();

    robius_url_handler::register_handler(|incoming_url| {
        use std::io::Write;

        // Note: here is where the URL should be acted upon.
        // Currently, we just log it to a temp file to prove that it works.
        let tmp = std::env::temp_dir();
        let now = std::time::SystemTime::now();
        std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(tmp.join("moly_incoming_url.txt"))
            .and_then(|mut f| {
                f.write_all(
                    format!("[{now:?}] Received incoming URL: {incoming_url:?}\n\n").as_bytes(),
                )
            })
            .unwrap();
    });

    tokio::runtime::Builder::new_multi_thread()
        // We are using non-tokio specific crates for fs and time operations,
        // but at least these needs to be enabled on native platforms using tokio
        // as their async runtime because `reqwest` use these.
        .enable_io()
        .enable_time()
        .build()
        .expect("Failed to create Tokio runtime")
        .block_on(async {
            // - Makepad's own event loop will block this main tokio thread.
            // - However, in some systems, UI apps can't run outside of the main thread,
            //   so using `spawn_blocking` may not work as expected.
            // - This forces us to at least use the multi-threaded runtime, requiring
            //   `Send` to spawn futures. However using single-threaded runtime
            //   would unify `Send` requirements across web and native.
            // - On web, `Send` should not be required as some crates interface with
            //   `wasm_bindgen` and `JsValue` is not `Send`.
            moly::app::app_main();
        })
}
