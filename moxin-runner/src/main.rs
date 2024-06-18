//! This `moxin-runner` application is a "companion" binary to the main `moxin` application.
//! This binary is reponsible for discovering the wasmedge installation,
//! installing wasmedge if it's missing, and setting up the environment properly
//! such that the main `moxin` app can locate the wasmedge dylibs and plugin dylibs.
//!
//! First, we discover the wasmedge installation.
//! The standard installation directory is `$HOME/.wasmedge`.
//! The default layout of the wasmedge installation directory is as follows:
//! ----------------------------------------------------
//! $HOME/.wasmedge
//! ├── bin
//! │   ├── wasmedge
//! │   └── wasmedgec
//! ├── env
//! ├── include
//! │   └── wasmedge
//! │       ├── enum.inc
//! │       ├── enum_configure.h
//! │       ├── enum_errcode.h
//! │       ├── enum_types.h
//! │       ├── int128.h
//! │       ├── version.h
//! │       └── wasmedge.h
//! ├── lib
//! │   ├── libwasmedge.0.0.3.dylib
//! │   ├── libwasmedge.0.0.3.tbd
//! │   ├── libwasmedge.0.dylib
//! │   ├── libwasmedge.0.tbd
//! │   ├── libwasmedge.dylib
//! │   └── libwasmedge.tbd
//! └── plugin
//!     ├── ggml-common.h
//!     ├── ggml-metal.metal
//!     ├── libwasmedgePluginWasiLogging.dylib
//!     └── libwasmedgePluginWasiNN.dylib
//! ----------------------------------------------------
//!
//! The key environment variables of interest are those that get set by the wasmedge installer.
//! 1. LIBRARY_PATH=$HOME/.wasmedge/lib
//! 2. C_INCLUDE_PATH=$HOME/.wasmedge/include
//! 3. CPLUS_INCLUDE_PATH=$HOME/.wasmedge/include
//!
//! Of these 3, we only care about the `LIBRARY_PATH`, where the `libwasmedge.0.dylib` is located.
//!
//! For loading plugins, we need to discover the plugin path. The plugin path can be set in the following ways:
//!/ * The environment variable "WASMEDGE_PLUGIN_PATH".
//!/ * The `../plugin/` directory related to the WasmEdge installation path.
//!/ * The `wasmedge/` directory under the library path if the WasmEdge is installed under the "/usr".
//!
//!
//! Moxin needs two wasmedge dylibs:
//! 1. the main `libwasmedge.0.dylib`,
//!    which is located in `$HOME/.wasmedge/lib/libwasmedge.0.dylib`.
//! 2. the wasi-nn plugin `libwasmedgePluginWasiNN.dylib`,
//!    which is located in `$HOME/.wasmedge/plugin/libwasmedgePluginWasiNN.dylib`.
//!

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub const MOXIN_APP_BINARY: &str = "_moxin_app";

const WASMEDGE_DIR_NAME: &str = ".wasmedge";
const LIB_DIR_NAME: &str = "lib";
const PLUGIN_DIR_NAME: &str = "plugin";

const WASMEDGE_MAIN_DYLIB: &str = {
    #[cfg(target_os = "macos")] {
        "libwasmedge.0.dylib"
    }
    #[cfg(target_os = "linux")] {
        "libwasmedge.so.0"
    }
};
const WASMEDGE_WASI_NN_PLUGIN_DYLIB: &str = {
    #[cfg(target_os = "macos")] {
        "libwasmedgePluginWasiNN.dylib"
    }
    #[cfg(target_os = "linux")] {
        "libwasmedgePluginWasiNN.so"
    }
};

const ENV_PATH: &str = "PATH";
const ENV_C_INCLUDE_PATH: &str = "C_INCLUDE_PATH";
const ENV_CPLUS_INCLUDE_PATH: &str = "CPLUS_INCLUDE_PATH";
const ENV_LIBRARY_PATH: &str = "LIBRARY_PATH";
const ENV_LD_LIBRARY_PATH: &str = {
    #[cfg(target_os = "macos")] {
        "DYLD_LIBRARY_PATH"
    }
    #[cfg(target_os = "linux")] {
        "LD_LIBRARY_PATH"
    }
};
#[cfg(target_os = "macos")]
const ENV_DYLD_FALLBACK_LIBRARY_PATH: &str = "DYLD_FALLBACK_LIBRARY_PATH";


/// An extension trait for checking if a path exists.
pub trait PathExt {
    fn path_if_exists(self) -> Option<Self> where Self: Sized;
}
impl<P: AsRef<Path>> PathExt for P {
    fn path_if_exists(self) -> Option<P> {
        if self.as_ref().as_os_str().is_empty() {
            return None;
        }
        match self.as_ref().try_exists() {
            Ok(true) => Some(self),
            _ => None,
        }
    }
}


fn main() -> std::io::Result<()> {
    let (wasmedge_dir_in_use, main_dylib_path, wasi_nn_plugin_path) = 
        // First, check if the wasmedge installation directory exists in the default location.
        existing_wasmedge_default_dir()
        // If not, try to find the wasmedge installation directory using environment vars.
        .or_else(wasmedge_dir_from_env_vars)
        // If we have a wasmedge installation directory, try to find the dylibs within it.
        .and_then(|wasmedge_dir| find_wasmedge_dylibs(&wasmedge_dir))
        // If we couldn't find the wasmedge directory or the dylibs within an existing directory,
        // then we must install wasmedge.
        .or_else(|| wasmedge_default_dir_path()
            .and_then(|default_path| install_wasmedge(default_path).ok())
            // If we successfully installed wasmedge, try to find the dylibs again.
            .and_then(find_wasmedge_dylibs)
        )
        .expect("failed to find or install wasmedge dylibs");

    println!("Found required wasmedge files:
        wasmedge root dir: {}
        wasmedge dylib:    {}
        wasi_nn plugin:    {}",
        wasmedge_dir_in_use.display(),
        main_dylib_path.display(),
        wasi_nn_plugin_path.display(),
    );

    apply_env_vars(&wasmedge_dir_in_use);
    run_moxin().unwrap();

    Ok(())
}


/// Returns an existing path to the default wasmedge installation directory, i.e., `$HOME/.wasmedge`,
/// but only if it exists.
fn existing_wasmedge_default_dir() -> Option<PathBuf> {
    wasmedge_default_dir_path().and_then(PathExt::path_if_exists)
}


/// Returns the path to where wasmedge is installed by default, i.e., `$HOME/.wasmedge`.
fn wasmedge_default_dir_path() -> Option<PathBuf> {
    directories::UserDirs::new()
        .map(|user_dirs| user_dirs.home_dir().join(WASMEDGE_DIR_NAME))
}


/// Looks for the wasmedge dylib and wasi_nn plugin dylib in the given `wasmedge_dir`.
///
/// The `wasmedge_dir` should be the root directory of the wasmedge installation;
/// see the crate-level documentation for more information about the expected layout.
/// 
/// Returns a tuple of:
/// 1. the wasmedge root directory path
/// 2. the main wasmedge dylib path
/// 3. the wasi_nn plugin dylib path
/// if found in `wasmedge_dir/lib/` and `wasmedge_dir/plugin/`.
fn find_wasmedge_dylibs<P: AsRef<Path>>(wasmedge_dir: P) -> Option<(PathBuf, PathBuf, PathBuf)> {
    let main_dylib_path = wasmedge_dir.as_ref()
        .join(LIB_DIR_NAME)
        .join(WASMEDGE_MAIN_DYLIB)
        .path_if_exists()?;
    let wasi_nn_plugin_path = wasmedge_dir.as_ref()
        .join(PLUGIN_DIR_NAME)
        .join(WASMEDGE_WASI_NN_PLUGIN_DYLIB)
        .path_if_exists()?;

    Some((wasmedge_dir.as_ref().into(), main_dylib_path, wasi_nn_plugin_path))
}


/// Installs wasmedge by downloading and running the wasmedge `install_v2.sh` script.
///
/// This function basically does the equivalent of running the following shell commands:
/// ```sh
/// curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash -s -- --path="<install_path>" --tmpdir="<std::env::temp_dir()>"
///
/// source $HOME/.wasmedge/env
/// ```
fn install_wasmedge<P: AsRef<Path>>(install_path: P) -> Result<P, std::io::Error> {
    println!("Attempting to install wasmedge to: {}", install_path.as_ref().display());
    let temp_dir = std::env::temp_dir();
    let curl_script_cmd = Command::new("curl")
        .arg("-s")
        .arg("-S")
        .arg("-f")
        .arg("https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh")
        .stdout(Stdio::piped())
        .spawn()?;

    let bash_cmd = Command::new("bash")
        .arg("-s")
        .arg("--")
        .arg(&format!("--path={}", install_path.as_ref().display()))
        // The default `/tmp/` dir used in `install_v2.sh` isn't always accessible to bundled apps.
        .arg(&format!("--tmpdir={}", temp_dir.display()))
        .stdin(Stdio::from(curl_script_cmd.stdout.expect("failed to pipe curl stdout into bash stdin")))
        .spawn()?;

    let output = bash_cmd.wait_with_output()?;
    if !output.status.success() {
        eprintln!("Failed to install wasmedge: {}
            ------------------------- stderr: -------------------------
            {:?}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        );
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "The wasmedge install_v2.sh script failed."));
    }

    println!("Successfully installed wasmedge to: {}", install_path.as_ref().display());

    apply_env_vars(&install_path);

    Ok(install_path)
} 


/// Applies the environment variable changes defined by `wasmedge_dir/env`.
///
/// The `wasmedge_dir` should be the root directory of the wasmedge installation,
/// which is typically `$HOME/.wasmedge`.
///
/// This does the following:
/// * Prepends `wasmedge_dir/bin` to `PATH`.
/// * Prepends `wasmedge_dir/lib` to `DYLD_LIBRARY_PATH`, `DYLD_FALLBACK_LIBRARY_PATH`, and `LIBRARY_PATH`.
/// * Prepends `wasmedge_dir/include` to `C_INCLUDE_PATH` and `CPLUS_INCLUDE_PATH`.
///
/// Note that we cannot simply run something like `Command::new("source")...`,
/// because `source` is a shell builtin, and the environment changes would only be visible
/// within that new process's shell instance -- not to this program.
fn apply_env_vars<P: AsRef<Path>>(wasmedge_dir_path: &P) {
    /// Prepends the given `prefix` to the environment variable with the given `key`.
    ///
    /// If the environment variable `key` is not set, it is set to the `prefix` value alone.
    fn prepend_env_var(env_key: impl AsRef<OsStr>, prefix: impl AsRef<OsStr>) {
        let key = env_key.as_ref();
        if let Some(existing) = std::env::var_os(key) {
            let mut joined_path = std::env::join_paths([prefix.as_ref(), OsStr::new("")]).unwrap();
            joined_path.push(&existing);
            std::env::set_var(key, joined_path);
        } else {
            std::env::set_var(key, prefix.as_ref());
        }
    }

    let wasmedge_dir = wasmedge_dir_path.as_ref();
    prepend_env_var(ENV_PATH, wasmedge_dir.join("bin"));
    prepend_env_var(ENV_C_INCLUDE_PATH, wasmedge_dir.join("include"));
    prepend_env_var(ENV_CPLUS_INCLUDE_PATH, wasmedge_dir.join("include"));
    prepend_env_var(ENV_LIBRARY_PATH, wasmedge_dir.join("lib"));
    prepend_env_var(ENV_LD_LIBRARY_PATH, wasmedge_dir.join("lib"));

    // The DYLD_FALLBACK_LIBRARY_PATH is only used on macOS.
    #[cfg(target_os = "macos")]
    prepend_env_var(ENV_DYLD_FALLBACK_LIBRARY_PATH, wasmedge_dir.join("lib"));

    // For macOS app bundles, we need to explicitly set the Plugin path to point to the Frameworks directory
    // inside the app bundle, where the plugin dylibs are located (they have been packaged alongside the app,
    // which is required on macOS for an app to be notarizable).
    #[cfg(target_os = "macos")]
    prepend_env_var("WASMEDGE_PLUGIN_PATH", "../Frameworks");
}


/// Attempts to discover the wasmedge installation directory using environment variables.
fn wasmedge_dir_from_env_vars() -> Option<PathBuf> {
    std::env::var_os(ENV_LD_LIBRARY_PATH)
        .or_else(|| std::env::var_os(ENV_LIBRARY_PATH))
        .or_else(|| std::env::var_os(ENV_C_INCLUDE_PATH))
        .or_else(|| std::env::var_os(ENV_CPLUS_INCLUDE_PATH))
        .and_then(|lib_path| PathBuf::from(lib_path)
            // All four of the above environment variables should point to a child directory
            // (either `lib/` or `include/`) within the wasmedge root directory.
            .parent()
            .and_then(PathExt::path_if_exists)
            .map(ToOwned::to_owned)
        )
}

/// Runs the `_moxin_app` binary, which must be located in the same directory as this moxin-runner binary.
fn run_moxin() -> std::io::Result<()> {
    let current_exe = std::env::current_exe()?;
    let current_exe_dir = current_exe.parent().unwrap();
    
    println!("------------------------- Environment Variables -------------------------");
    println!("{:#?}", std::env::vars().collect::<Vec<_>>());
    println!("Running moxin in dir: {}", current_exe_dir.display());

    let _output = Command::new(current_exe_dir.join(MOXIN_APP_BINARY))
        .current_dir(current_exe_dir)
        .spawn()?
        .wait_with_output()?;

    Ok(())
}
