use std::{env, fs, io, thread};
use mega;
use std::io::Write;
use moly::data::filesystem::project_dirs;

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
    run_mega_server();
    moly::app::app_main()
}

/// Start the Mega server in a separate thread.
fn run_mega_server() {
    env::set_var("MEGA_BASE_DIR", project_dirs().data_dir().join(".mega").to_str().unwrap());
    thread::spawn(|| -> io::Result<()> {
        let config = include_str!("../.mega/config.toml"); // save config as String (soft-hard code)
        let config_path = project_dirs().config_dir().join(".mega/config.toml");
        if !config_path.exists() {
            fs::create_dir_all(config_path.parent().unwrap())?;
            let mut file = fs::File::create(&config_path)?;
            file.write_all(config.as_bytes())?;
        }

        let args = vec!["-c", config_path.to_str().unwrap(), "service", "multi", "http", "--bootstrap-node", "http://gitmono.org/relay"];
        println!("Starting Mega with args: {:?}", args);
        mega::cli::parse(Some(args)).expect("Failed to start Mega");
        Ok(())
    });
}
