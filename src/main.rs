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
#[tokio::main]
async fn main() {
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

    moly::app::app_main()
}
