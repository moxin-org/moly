use std::io::Write;

fn main() {
    robius_url_handler::register_handler(|incoming_url| {
        // Note: here is where the URL should be acted upon.
        // Currently, we just log it to a temp file to prove that it works.
        let tmp = std::env::temp_dir();
        let now = std::time::SystemTime::now();
        std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(tmp.join("moly_incoming_url.txt"))
            .and_then(|mut f| 
                f.write_all(format!("[{now:?}] Received incoming URL: {incoming_url:?}\n\n").as_bytes())
            )
            .unwrap();
    });

    moly::app::app_main()
}
