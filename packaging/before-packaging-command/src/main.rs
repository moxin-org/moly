//! This small program is invoked by cargo-packager during its before-packaging steps.
//!
//! This program must be run from the root of the project directory,
//! which is also where the cargo-packager command must be invoked from.
//!
//! This program runs in two modes, one for each kind of before-packaging step in cargo-packager.
//! It accepts arguments to determine which mode:
//! * Argument `before-packaging`: specifies that the `before-packaging-command` is being run
//!   by cargo-packager, which gets executed only *once* before cargo-packager generates any package bundles.
//! * Argument `before-each-package`: specifies that the `before-each-package-command` is being run
//!   by cargo-packager, which gets executed multiple times: once for *each* package
//!   that cargo-packager is going to generate.
//!    * The environment variable `CARGO_PACKAGER_FORMAT` is set by cargo-packager to
//!      the declare which package format is about to be generated, which include the values
//!      given here: <https://docs.rs/cargo-packager/latest/cargo_packager/enum.PackageFormat.html>.
//!      * `app`, `dmg`: for macOS.
//!      * `deb`, `appimage`, `pacman`: for Linux.
//!      * `nsis`: for Windows; `nsis` generates an installer `setup.exe`.
//!      * `wix`: (UNSUPPORTED) for Windows; generates an `.msi` installer package.
//!
//! This program uses the `CARGO_PACKAGER_FORMAT` environment variable to determine
//! which specific build commands and configuration options should be used.
//!

use core::panic;
use std::{ffi::OsStr, fs, path::Path, process::{Command, Stdio}};
use cargo_metadata::MetadataCommand;


/// Returns the value of the `MAKEPAD_PACKAGE_DIR` environment variable
/// that must be set for the given package format.
///
/// * For macOS app bundles, this should be set to `../Resources`.
///   * This only works because the `moly-runner` binary sets the current working directory
///     to the directory where the binary is located, which is `Moly.app/Contents/MacOS/`.
///     (See the `run_moly` function in `moly-runner/src/main.rs` for more details.)
///   * In a macOS app bundle, the resources directory is in `Moly.app/Context/Resources/`,
///     so that's why we set `MAKEPAD_PACKAGE_DIR` to `../Resources`. 
///     This must be relative to the binary's location, i.e. up one parent directory.
/// * For AppImage packages, this should be set to a relative path that goes up two parent directories
///   to account for the fact that AppImage binaries run in a simulated `usr/bin/` directory.
///   * Thus, we need to get to the simulated `usr/lib/` directory for Moly's resources, 
///     which currently works out to `../../usr/lib/moly`.
///   * Note that this must be a relative path, not an absolute path.
/// * For Debian `.deb` packages, this should be set to `/usr/lib/<main-binary-name>`,
///   which is currently `/usr/lib/moly-runner`.
///   * This is the directory in which `dpkg` copies app resource files to
///     when a user installs the `.deb` package.
/// * For Windows NSIS packages, this should be set to `.` (the current dir).
///  * This is because the NSIS installer script copies the resources to the same directory
///    as the installed binaries, and our `moly-runner` app changes the current working directory
///    to that same directory before running the main Moly binary.
fn makepad_package_dir_value(package_format: &str) -> &'static str {
    match package_format {
        "app" | "dmg" => "../Resources",
        "appimage" => "../../usr/lib/moly",
        "deb" | "pacman" => "/usr/lib/moly",
        "nsis" => ".",
        _other => panic!("Unsupported package format: {}", _other),
    }
}


fn main() -> std::io::Result<()> {
    let mut is_before_packaging = false;
    let mut is_before_each_package = false;
    let mut host_os_opt: Option<String> = None;

    let mut args = std::env::args().peekable();
    while let Some(arg) = args.next() {
        if arg.ends_with("before-packaging") || arg.ends_with("before_packaging") {
            is_before_packaging = true;
        }
        if arg.contains("before-each") || arg.contains("before_each") {
            is_before_each_package = true;
        }
        if host_os_opt.is_none() && (arg.contains("host_os") || arg.contains("host-os"))
        {
            host_os_opt = arg
                .split("=")
                .last()
                .map(|s| s.to_string())
                .or_else(|| args.peek().map(|s| s.to_string()));
        }
    }

    let host_os = host_os_opt.as_deref().unwrap_or(std::env::consts::OS);

    match (is_before_packaging, is_before_each_package) {
        (true, false) => before_packaging(host_os),
        (false, true) => before_each_package(host_os),
        (true, true) => panic!("Cannot run both 'before-packaging' and 'before-each-package' commands at the same time."),
        (false, false) => panic!("Please specify either the 'before-packaging' or 'before-each-package' command."),
    }
}

/// This function is run only *once* before cargo-packager generates any package bundles.
///
/// ## Functionality
/// 1. Creates a directory for the resources to be packaged, which is currently `./dist/resources/`.
/// 2. Recursively copies the resources from the `makepad-widgets` crate to a subdirectory
///    of the directory created in step 1, which is currently `./dist/resources/makepad_widgets/`.
///    The location of the `makepad-widgets` crate is determined using `cargo-metadata`.
/// 3. Recursively copies the Moly-specific `./resources` directory to `./dist/resources/moly/`.
/// 4. (If macOS) Downloads WasmEdge and sets up the plugins into the `./wasmedge/` directory.
fn before_packaging(host_os: &str) -> std::io::Result<()> {
    let cwd = std::env::current_dir()?;
    let dist_resources_dir = cwd.join("dist").join("resources");
    fs::create_dir_all(&dist_resources_dir)?;

    let moly_resources_dest = dist_resources_dir.join("moly").join("resources");
    let moly_resources_src = cwd.join("resources");
    let makepad_widgets_resources_dest = dist_resources_dir.join("makepad_widgets").join("resources");
    let makepad_widgets_resources_src = {
        let cargo_metadata = MetadataCommand::new()
            .exec()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let makepad_widgets_package = cargo_metadata
            .packages
            .iter()
            .find(|package| package.name == "makepad-widgets")
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "makepad-widgets package not found"))?;

        makepad_widgets_package.manifest_path
            .parent()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "makepad-widgets package manifest path not found"))?
            .join("resources")
    };

    /// Copy files from source to destination recursively.
    fn copy_recursively(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> std::io::Result<()> {
        fs::create_dir_all(&destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let filetype = entry.file_type()?;
            if filetype.is_dir() {
                copy_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }

    println!("Copying makepad-widgets resources...\n  --> From: {}\n      to:   {}", makepad_widgets_resources_src.as_std_path().display(), makepad_widgets_resources_dest.display());
    copy_recursively(&makepad_widgets_resources_src, &makepad_widgets_resources_dest)?;
    println!("  --> Done!");
    println!("Copying moly resources...\n  --> From {}\n      to:   {}", moly_resources_src.display(), moly_resources_dest.display());
    copy_recursively(&moly_resources_src, &moly_resources_dest)?;
    println!("  --> Done!");

    if host_os == "macos" {
       download_wasmedge_macos("0.14.1")?;
    }

    Ok(())
}


/// Downloads WasmEdge and extracts it to the `./wasmedge/` directory.
///
/// This function effectively runs the following shell commands:
/// ```sh
///    mkdir -p ./wasmedge \
///    && curl -sfL --show-error https://github.com/WasmEdge/WasmEdge/releases/download/0.14.0/WasmEdge-0.14.0-darwin_arm64.tar.gz | bsdtar -xf- -C ./wasmedge \
///    && mkdir -p ./wasmedge/WasmEdge-0.14.0-Darwin/plugin \
///    && curl -sf --location --progress-bar --show-error https://github.com/WasmEdge/WasmEdge/releases/download/0.14.0/WasmEdge-plugin-wasi_nn-ggml-0.14.0-darwin_arm64.tar.gz | bsdtar -xf- -C ./wasmedge/WasmEdge-0.14.0-Darwin/plugin; \
/// ```
fn download_wasmedge_macos(version: &str) -> std::io::Result<()> {
    // Command 1: Create the destination directory.
    let dest_dir = std::env::current_dir()?.join("wasmedge");
    fs::create_dir_all(&dest_dir)?;

    // Command 2: Download and extract WasmEdge.
    {
        println!("Downloading wasmedge v{version} to: {}", dest_dir.display());
        let curl_script_cmd = Command::new("curl")
            .arg("-sSfL")
            .arg("https://github.com/WasmEdge/WasmEdge/releases/download/0.14.0/WasmEdge-0.14.0-darwin_arm64.tar.gz")
            .stdout(Stdio::piped())
            .spawn()?;

        let bsdtar_cmd = Command::new("bsdtar")
            .arg("-xf-")
            .arg("-C")
            .arg(&dest_dir)
            .stdin(Stdio::from(curl_script_cmd.stdout.expect("failed to pipe curl stdout into bsdtar stdin")))
            .spawn()?;

        let output = bsdtar_cmd.wait_with_output()?;
        if !output.status.success() {
            eprintln!("Failed to install WasmEdge: {}
                ------------------------- stderr: -------------------------
                {:?}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
            );
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "The wasmedge install_v2.sh script failed."));
        }
    }

    // Command 3: Create the plugin destination directory.
    let plugin_dest_dir = dest_dir.join("WasmEdge-0.14.0-Darwin").join("plugin");
    fs::create_dir_all(&plugin_dest_dir)?;
    
    // Command 4: Download and extract the Wasi-NN plugin.
    {
        println!("Downloading wasmedge v{version} WASI-NN plugin to: {}", plugin_dest_dir.display());
        let curl_script_cmd = Command::new("curl")
            .arg("-sSfL")
            .arg("https://github.com/WasmEdge/WasmEdge/releases/download/0.14.0/WasmEdge-plugin-wasi_nn-ggml-0.14.0-darwin_arm64.tar.gz")
            .stdout(Stdio::piped())
            .spawn()?;

        let bsdtar_cmd = Command::new("bsdtar")
            .arg("-xf-")
            .arg("-C")
            .arg(&plugin_dest_dir)
            .stdin(Stdio::from(curl_script_cmd.stdout.expect("failed to pipe curl stdout into bsdtar stdin")))
            .spawn()?;

        let output = bsdtar_cmd.wait_with_output()?;
        if !output.status.success() {
            eprintln!("Failed to install WasmEdge Wasi-NN plugin: {}
                ------------------------- stderr: -------------------------
                {:?}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
            );
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "The wasmedge install_v2.sh script failed."));
        }
    }

    Ok(())
}


/// The function that is run by cargo-packager's `before-each-package-command`.
///
/// It's just a simple wrapper that invokes the function for each specific package format.
fn before_each_package(host_os: &str) -> std::io::Result<()> {
    // The `CARGO_PACKAGER_FORMAT` environment variable is required.
    let format = std::env::var("CARGO_PACKAGER_FORMAT")
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let package_format = format.as_str();
    println!("Running before-each-package-command for {package_format:?}");
    match package_format         {
        "app" | "dmg" => before_each_package_macos(package_format, host_os),
        "deb"         => before_each_package_deb(package_format, host_os),
        "appimage"    => before_each_package_appimage(package_format, host_os),
        "pacman"      => before_each_package_pacman(package_format, host_os),
        "nsis"        => before_each_package_windows(package_format, host_os),
        _other => return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Unknown/unsupported package format {_other:?}"),
        )),
    }
}


/// Runs the macOS-specific build commands for "app" and "dmg" package formats.
///
/// This function effectively runs the following shell commands:
/// ```sh
///    MAKEPAD_PACKAGE_DIR=../Resources  cargo build --workspace --release --features macos_bundle \
///    && install_name_tool -add_rpath "@executable_path/../Frameworks" ./target/release/_moly_app;
/// ```
fn before_each_package_macos(package_format: &str, host_os: &str) -> std::io::Result<()> {
    assert!(host_os == "macos", "'app' and 'dmg' packages can only be created on macOS.");

    cargo_build(
        package_format,
        host_os,
        &["--features", "macos_bundle"],
    )?;

    // Use `install_name_tool` to add the `@executable_path` rpath to the binary.
    let install_name_tool_cmd = Command::new("install_name_tool")
        .arg("-add_rpath")
        .arg("@executable_path/../Frameworks")
        .arg("./target/release/_moly_app")
        .spawn()?;

    let output = install_name_tool_cmd.wait_with_output()?;
    if !output.status.success() {
        eprintln!("Failed to run install_name_tool command: {}
            ------------------------- stderr: -------------------------
            {:?}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        );
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to run install_name_tool command for macOS"));
    }

    Ok(())

}

/// Runs the Linux-specific build commands for AppImage packages.
fn before_each_package_appimage(package_format: &str, host_os: &str) -> std::io::Result<()> {
    assert!(host_os == "linux", "AppImage packages can only be created on Linux.");

    cargo_build(
        package_format,
        host_os,
        std::iter::empty::<&str>(),
    )?;

    strip_unneeded_linux_binaries(host_os)?;

    Ok(())
}


/// Runs the Linux-specific build commands for Debian `.deb` packages.
///
/// This function effectively runs the following shell commands:
/// ```sh
///    for path in $(ldd target/release/_moly_app | awk '{print $3}'); do \
///        basename "$/path" ; \
///    done \
///    | xargs dpkg -S 2> /dev/null | awk '{print $1}' | awk -F ':' '{print $1}' | sort | uniq > ./dist/depends_deb.txt; \
///    echo "curl" >> ./dist/depends_deb.txt; \
///    
fn before_each_package_deb(package_format: &str, host_os: &str) -> std::io::Result<()> {
    assert!(host_os == "linux", "'deb' packages can only be created on Linux.");

    cargo_build(
        package_format,
        host_os,
        &["--features", "reqwest/native-tls-vendored"],
    )?;

    
    // Create Debian dependencies file by running `ldd` on the binary
    // and then running `dpkg -S` on each unique shared libraries outputted by `ldd`.
    let ldd_output = Command::new("ldd")
        .arg("target/release/_moly_app")
        .output()?;

    let ldd_output = if ldd_output.status.success() {
        String::from_utf8_lossy(&ldd_output.stdout)
    } else {
        eprintln!("Failed to run ldd command: {}
            ------------------------- stderr: -------------------------
            {:?}",
            ldd_output.status,
            String::from_utf8_lossy(&ldd_output.stderr),
        );
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to run ldd command on {host_os} for package format {package_format:?}")
        ));
    };

    let mut dpkgs = Vec::new();
    for line_raw in ldd_output.lines() {
        let line = line_raw.trim();
        let lib_name_opt = line.split_whitespace()
            .next()
            .and_then(|path| Path::new(path)
                .file_name()
                .and_then(|f| f.to_str().to_owned())
            );
        let Some(lib_name) = lib_name_opt else { continue };

        let dpkg_output = Command::new("dpkg")
            .arg("-S")
            .arg(lib_name)
            .stderr(Stdio::null())
            .output()?;
        let dpkg_output = if dpkg_output.status.success() {
            String::from_utf8_lossy(&dpkg_output.stdout)
        } else {
            // Skip shared libraries that dpkg doesn't know about, e.g., `linux-vdso.so*`
            continue;
        };

        let Some(package_name) = dpkg_output.split(':').next() else { continue };
        println!("Got dpkg dependency {package_name:?} from ldd output: {line:?}");
        dpkgs.push(package_name.to_string());
    }
    dpkgs.sort();
    dpkgs.dedup();
    println!("Sorted and de-duplicated dependencies: {:#?}", dpkgs);
    // `curl` is a fixed dependency for the moly-runner binary.
    dpkgs.push("curl".to_string());
    std::fs::write("./dist/depends_deb.txt", dpkgs.join("\n"))?;
    
    strip_unneeded_linux_binaries(host_os)?;
    Ok(())
}


/// Runs the Linux-specific build commands for PacMan packages.
///
/// This is untested and may be incomplete, e.g., dependencies are not determined.
fn before_each_package_pacman(package_format: &str, host_os: &str) -> std::io::Result<()> {
    assert!(host_os == "linux", "Pacman packages can only be created on Linux.");

    cargo_build(
        package_format,
        host_os,
        &["--features", "reqwest/native-tls-vendored"],
    )?;

    strip_unneeded_linux_binaries(host_os)?;
    Ok(())
}
    
/// Runs the Windows-specific build commands for WiX (`.msi`) and NSIS (`.exe`) packages.
fn before_each_package_windows(package_format: &str, host_os: &str) -> std::io::Result<()> {
    assert!(host_os == "windows", "'.exe' and '.msi' packages can only be created on Windows.");

    cargo_build(
        package_format,
        host_os,
        std::iter::empty::<&str>(),
    )?;

    Ok(())
}

fn cargo_build<I, S>(package_format: &str, _host_os: &str, extra_args: I) -> std::io::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let cargo_build_cmd = Command::new("cargo")
        .arg("build")
        .arg("--workspace")
        .arg("--release")
        .args(extra_args)
        .env("MAKEPAD_PACKAGE_DIR", makepad_package_dir_value(package_format))
        .spawn()?;

    let output = cargo_build_cmd.wait_with_output()?;
    if !output.status.success() {
        eprintln!("Failed to run cargo build command: {}
            ------------------------- stderr: -------------------------
            {:?}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        );
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to run cargo build command on {_host_os} for package format {package_format:?}")
        ));
    }

    Ok(())
}
/// Strips unneeded symbols from the Linux binary, which is required for Debian `.deb` packages
/// and recommended for all other Linux package formats.
fn strip_unneeded_linux_binaries(host_os: &str) -> std::io::Result<()> {
    assert!(host_os == "linux", "'strip --strip-unneeded' can only be run on Linux.");
    let strip_cmd = Command::new("strip")
        .arg("--strip-unneeded")
        .arg("--remove-section=.comment")
        .arg("--remove-section=.note")
        .arg("target/release/_moly_app")
        .arg("target/release/moly")
        .spawn()?;

    let output = strip_cmd.wait_with_output()?;
    if !output.status.success() {
        eprintln!("Failed to run strip command: {}
            ------------------------- stderr: -------------------------
            {:?}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        );
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to run strip command for Linux"));
    }

    Ok(())
}
