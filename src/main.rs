use std::{env, thread};
use mega;
use std::io::Write;
use moxin::data::filesystem::project_dirs;

fn main() {
    robius_url_handler::register_handler(|incoming_url| {
        // Note: here is where the URL should be acted upon.
        // Currently, we just log it to a temp file to prove that it works.
        let tmp = std::env::temp_dir();
        let now = std::time::SystemTime::now();
        std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(tmp.join("moxin_incoming_url.txt"))
            .and_then(|mut f| 
                f.write_all(format!("[{now:?}] Received incoming URL: {incoming_url:?}\n\n").as_bytes())
            )
            .unwrap();
    });
    run_mega_server();
    moxin::app::app_main()
}

/// Start the Mega server in a separate thread.
fn run_mega_server() {
    env::set_var("MEGA_BASE_DIR", project_dirs().data_dir().join(".mega").to_str().unwrap());
    thread::spawn(|| {
        println!("Mega Started");
        let args = "service multi http".split(' ').collect();
        mega::cli::parse(Some(args)).expect("Failed to start Mega");
    });
}
