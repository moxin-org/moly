//! This `moxin-runner` application is a "companion" binary to the main `moxin` application.
//! This binary is reponsible for discovering the wasmedge installation,
//! installing wasmedge if it's missing, and setting up the environment properly
//! such that the main `moxin` app can locate the wasmedge dylibs and plugin dylibs.
//!
//! First, we discover the wasmedge installation.
//! * The standard installation directory on macOS and Linux is `$HOME/.wasmedge`.
//! * On macOS, the default layout of the wasmedge installation directory is as follows:
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
//! 1. WASMEDGE_DIR=$HOME/.wasmedge
//! 2. LIBRARY_PATH=$HOME/.wasmedge/lib
//! 3. C_INCLUDE_PATH=$HOME/.wasmedge/include
//! 4. CPLUS_INCLUDE_PATH=$HOME/.wasmedge/include
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
//! On Windows and Linux, the concepts are the same, but the file names and 
//! directory layout of WasmEdge differ from macOS.
//!

#![cfg_attr(feature = "macos_bundle", allow(unused))]

use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub const MOXIN_APP_BINARY: &str = "_moxin_app";

/// The name of the wasmedge installation directory.
const WASMEDGE_ROOT_DIR_NAME: &str = {
    #[cfg(any(target_os = "linux", target_os = "macos"))] {
        ".wasmedge"
    }
    #[cfg(windows)] {
        "WasmEdge-0.14.0-Windows"
    }
};

/// The subdirectory within the WasmEdge root directory where the main dylib is located.
const DYLIB_DIR_NAME: &str = {
    #[cfg(any(target_os = "linux", target_os = "macos"))] {
        "lib"
    }
    #[cfg(windows)] {
        "bin"
    }
};


/// The subdirectory within the WasmEdge root directory where the plugin dylibs are located.
fn plugin_dir_path_from_root() -> PathBuf {
    #[cfg(any(target_os = "linux", target_os = "macos"))] {
        PathBuf::from("plugin")
    }
    #[cfg(windows)] {
        Path::new("lib").join("wasmedge")
    }
}

/// The file name of the main WasmEdge dylib.
const WASMEDGE_MAIN_DYLIB: &str = {
    #[cfg(target_os = "macos")] {
        "libwasmedge.0.dylib"
    }
    #[cfg(target_os = "linux")] {
        "libwasmedge.so.0"
    }
    #[cfg(windows)] {
        "wasmedge.dll"
    }
};
/// The file name of the Wasi-NN plugin dylib.
const WASMEDGE_WASI_NN_PLUGIN_DYLIB: &str = {
    #[cfg(target_os = "macos")] {
        "libwasmedgePluginWasiNN.dylib"
    }
    #[cfg(target_os = "linux")] {
        "libwasmedgePluginWasiNN.so"
    }
    #[cfg(windows)] {
        "wasmedgePluginWasiNN.dll"
    }
};

const ENV_WASMEDGE_DIR: &str = "WASMEDGE_DIR";
#[allow(unused)]
const ENV_WASMEDGE_PLUGIN_PATH: &str = "WASMEDGE_PLUGIN_PATH";
const ENV_PATH: &str = "PATH";
const ENV_C_INCLUDE_PATH: &str = "C_INCLUDE_PATH";
const ENV_CPLUS_INCLUDE_PATH: &str = "CPLUS_INCLUDE_PATH";
const ENV_LIBRARY_PATH: &str = "LIBRARY_PATH";
#[cfg(target_os = "macos")]
const ENV_LD_LIBRARY_PATH: &str = "DYLD_LIBRARY_PATH";
#[cfg(target_os = "linux")]
const ENV_LD_LIBRARY_PATH: &str = "LD_LIBRARY_PATH";
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


#[cfg(feature = "macos_bundle")]
fn main() -> std::io::Result<()> {
    // For macOS app bundles, the WasmEdge dylibs have already been packaged inside of the app bundle,
    // specifically in the `Contents/Frameworks/` subdirectory.
    // This is required for the app bundle to be notarizable.
    //
    // Thus, we don't need to discover, locate, or install wasmedge.
    // We only need to explicitly set the wasmedge lugin path to point to the `Frameworks/` directory
    // inside the app bundle, which is within the parent directory of the executables in the app bundle.
    //
    // Thus, we set the `WASMEDGE_PLUGIN_PATH` environment variable to `../Frameworks`,
    // because the run_moxin() function will set the current working directory to `Contents/MacOS/`
    // within the app bundle, which is the subdirectory that contains the actual moxin executables.
    std::env::set_var(ENV_WASMEDGE_PLUGIN_PATH, "../Frameworks");

    println!("Running within a macOS app bundle.
        {ENV_WASMEDGE_PLUGIN_PATH}: {:?}",
        std::env::var(ENV_WASMEDGE_PLUGIN_PATH).ok()
    );

    run_moxin(None).unwrap();
    Ok(())
}


#[cfg(not(feature = "macos_bundle"))]
fn main() -> std::io::Result<()> {
    check_cpu_features();

    let (wasmedge_root_dir_in_use, main_dylib_path, wasi_nn_plugin_path) = 
        // First, try to find the wasmedge installation directory using environment vars.
        wasmedge_root_dir_from_env_vars()
        // If not, check if the wasmedge installation directory exists in the default location.
        .or_else(existing_wasmedge_default_dir)
        // If we have a wasmedge installation directory, try to find the dylibs within it.
        .and_then(|wasmedge_root_dir| find_wasmedge_dylibs_in_dir(&wasmedge_root_dir))
        // If we couldn't find the wasmedge directory or the dylibs within an existing directory,
        // then we must install wasmedge.
        .or_else(|| wasmedge_default_dir_path()
            .and_then(|default_path| install_wasmedge(default_path).ok())
            // If we successfully installed wasmedge, try to find the dylibs again.
            .and_then(find_wasmedge_dylibs_in_dir)
        )
        .expect("failed to find or install wasmedge dylibs");

    println!("Found required wasmedge files:
        wasmedge root dir: {}
        wasmedge dylib:    {}
        wasi_nn plugin:    {}",
        wasmedge_root_dir_in_use.display(),
        main_dylib_path.display(),
        wasi_nn_plugin_path.display(),
    );

    apply_env_vars(&wasmedge_root_dir_in_use);

    run_moxin(main_dylib_path.parent())
}


/// Returns the path to the default wasmedge installation directory, if it exists.
fn existing_wasmedge_default_dir() -> Option<PathBuf> {
    wasmedge_default_dir_path()?.path_if_exists()
}


/// Returns the path to where wasmedge is installed by default.
///
/// This does not check if the directory actually exists.
fn wasmedge_default_dir_path() -> Option<PathBuf> {
    directories::UserDirs::new()
        .map(|dirs| dirs.home_dir().join(WASMEDGE_ROOT_DIR_NAME))
}


/// Looks for the wasmedge dylib and wasi_nn plugin dylib in the given `wasmedge_root_dir`.
///
/// The `wasmedge_root_dir` should be the root directory of the wasmedge installation;
/// see the crate-level documentation for more information about the expected layout.
/// 
/// If all items were found in their expected locations, this returns a tuple of:
/// 1. the wasmedge root directory path,
/// 2. the main wasmedge dylib path,
/// 3. the wasi_nn plugin dylib path.
fn find_wasmedge_dylibs_in_dir<P: AsRef<Path>>(wasmedge_root_dir: P) -> Option<(PathBuf, PathBuf, PathBuf)> {
    let main_dylib_path = wasmedge_root_dir.as_ref()
        .join(DYLIB_DIR_NAME)
        .join(WASMEDGE_MAIN_DYLIB)
        .path_if_exists()?;
    let wasi_nn_plugin_path = wasmedge_root_dir.as_ref()
        .join(plugin_dir_path_from_root())
        .join(WASMEDGE_WASI_NN_PLUGIN_DYLIB)
        .path_if_exists()?;

    Some((wasmedge_root_dir.as_ref().into(), main_dylib_path, wasi_nn_plugin_path))
}


/// Installs wasmedge by downloading and running the wasmedge `install_v2.sh` script.
///
/// This function basically does the equivalent of running the following shell commands:
/// ```sh
/// curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash -s -- --path="<install_path>" --tmpdir="<std::env::temp_dir()>"
///
/// source $HOME/.wasmedge/env
/// ```
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn install_wasmedge<P: AsRef<Path>>(install_path: P) -> Result<PathBuf, std::io::Error> {
    use std::process::Stdio;
    println!("Downloading WasmEdge 0.14.0 from GitHub and installing it to {}", install_path.as_ref().display());
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
        .arg("--version=0.14.0")
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

    Ok(install_path.as_ref().to_path_buf())
} 


/// Installs WasmEdge by calling out to PowerShell to run the Windows installation steps
/// provided in the main Moxin README.
///
/// The given `install_path` is currently ignored, using the [wasmedge_default_dir_path()] instead.
///
/// The PowerShell script we run simply downloads and extracts the main WasmEdge files and the Wasi-NN plugin.
/// ```powershell
///    Invoke-WebRequest -Uri "https://github.com/WasmEdge/WasmEdge/releases/download/0.14.0/WasmEdge-0.14.0-windows.zip" -OutFile "$env:TEMP\WasmEdge-0.14.0-windows.zip"
///    Expand-Archive -Force -Path "$env:TEMP\WasmEdge-0.14.0-windows.zip" -DestinationPath $home
///    Invoke-WebRequest -Uri "https://github.com/WasmEdge/WasmEdge/releases/download/0.14.0/WasmEdge-plugin-wasi_nn-ggml-0.14.0-windows_x86_64.zip" -OutFile "$env:TEMP\WasmEdge-plugin-wasi_nn-ggml-0.14.0-windows_x86_64.zip"
///    Expand-Archive -Force -Path "$env:TEMP\WasmEdge-plugin-wasi_nn-ggml-0.14.0-windows_x86_64.zip" -DestinationPath "$home\WasmEdge-0.14.0-Windows"
/// ```
#[cfg(windows)]
fn install_wasmedge<P: AsRef<Path>>(_install_path: P) -> Result<PathBuf, std::io::Error> {
    println!("Downloading and installing WasmEdge 0.14.0 from GitHub.");
    let install_wasmedge_ps1 = include_str!("powershell_install_wasmedge.ps1");
    match powershell_script::PsScriptBuilder::new()
        .non_interactive(true)
        .hidden(true) // Don't display a PowerShell window
        .print_commands(false) // enable this for debugging
        .build()
        .run(&install_wasmedge_ps1)
    {
        Ok(output) => {
            if output.success() {
                // The wasmedge installation directory is currently forced to the default dir path.
                wasmedge_default_dir_path().ok_or_else(
                || std::io::Error::new(std::io::ErrorKind::Other, "BUG: couldn't get WasmEdge default directory path.")
                )
            } else {
                eprintln!("------------- Powershell stdout --------------\n{}", output.stdout().unwrap_or_default());
                eprintln!("----------------------------------------------\n");
                eprintln!("------------- Powershell stderr --------------\n{}", output.stderr().unwrap_or_default());
                eprintln!("----------------------------------------------\n");
                Err(std::io::Error::new(std::io::ErrorKind::Other, "The WasmEdge install PowerShell script failed."))
            }
        }
        Err(err) => {
            eprintln!("Failed to install wasmedge: {:?}", err);
            if let powershell_script::PsError::Powershell(output) = err {
                eprintln!("------------- Powershell stdout --------------\n{}", output.stdout().unwrap_or_default());
                eprintln!("----------------------------------------------\n");
                eprintln!("------------- Powershell stderr --------------\n{}", output.stderr().unwrap_or_default());
                eprintln!("----------------------------------------------\n");
            }
            Err(std::io::Error::new(std::io::ErrorKind::Other, "The WasmEdge install PowerShell script failed."))
        }
    }
}


/// Applies the environment variable changes defined by `wasmedge_root_dir/env`.
///
/// The `wasmedge_root_dir` should be the root directory of the wasmedge installation,
/// which is typically `$HOME/.wasmedge`.
///
/// This does the following:
/// * Prepends `wasmedge_root_dir/bin` to `PATH`.
/// * Prepends `wasmedge_root_dir/lib` to `DYLD_LIBRARY_PATH`, `DYLD_FALLBACK_LIBRARY_PATH`, and `LIBRARY_PATH`.
/// * Prepends `wasmedge_root_dir/include` to `C_INCLUDE_PATH` and `CPLUS_INCLUDE_PATH`.
///
/// Note that we cannot simply run something like `Command::new("source")...`,
/// because `source` is a shell builtin, and the environment changes would only be visible
/// within that new process's shell instance -- not to this program.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn apply_env_vars<P: AsRef<Path>>(wasmedge_root_dir_path: &P) {
    use std::ffi::OsStr;
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

    let wasmedge_root_dir = wasmedge_root_dir_path.as_ref();
    prepend_env_var(ENV_PATH, wasmedge_root_dir.join("bin"));
    prepend_env_var(ENV_C_INCLUDE_PATH, wasmedge_root_dir.join("include"));
    prepend_env_var(ENV_CPLUS_INCLUDE_PATH, wasmedge_root_dir.join("include"));
    prepend_env_var(ENV_LIBRARY_PATH, wasmedge_root_dir.join("lib"));
    prepend_env_var(ENV_LD_LIBRARY_PATH, wasmedge_root_dir.join("lib"));

    // The DYLD_FALLBACK_LIBRARY_PATH is only used on macOS.
    #[cfg(target_os = "macos")]
    prepend_env_var(ENV_DYLD_FALLBACK_LIBRARY_PATH, wasmedge_root_dir.join("lib"));
}


/// Applies the environment variables needed for Moxin to find WasmEdge on Windows.
///
/// Currently, this only does the following:
/// * Sets [ENV_WASMEDGE_DIR] and [ENV_WASMEDGE_PLUGIN_PATH] to the given `wasmedge_root_dir_path`.
#[cfg(windows)]
fn apply_env_vars<P: AsRef<Path>>(wasmedge_root_dir_path: &P) {
    std::env::set_var(ENV_WASMEDGE_DIR, wasmedge_root_dir_path.as_ref());
    std::env::set_var(ENV_WASMEDGE_PLUGIN_PATH, wasmedge_root_dir_path.as_ref());
}

/// Attempts to discover the wasmedge installation directory using environment variables.
///
/// * On Windows, only the [ENV_WASMEDGE_DIR] environment variable can be used.
/// * On Linux and macOS, all other environment variables are checked.
fn wasmedge_root_dir_from_env_vars() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(ENV_WASMEDGE_DIR).and_then(PathExt::path_if_exists) {
        return Some(dir.into());
    }
    // Note: we cannot use ENV_WASMEDGE_PLUGIN_PATH here, because it can point to multiple directories, 
    // e.g., the wasmedge root dir, or one of the subdirectories within it.

    #[cfg(any(target_os = "linux", target_os = "macos"))] {
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
    #[cfg(windows)] {
        None
    }
}

/// Runs the `_moxin_app` binary, which must be located in the same directory as this moxin-runner binary.
///
/// An optional path to the directory containing the main WasmEdge dylib can be provided,
/// which is currently only used to set the path on Windows.
fn run_moxin(_main_wasmedge_dylib_dir: Option<&Path>) -> std::io::Result<()> {
    let current_exe = std::env::current_exe()?;
    let current_exe_dir = current_exe.parent().unwrap();
    let args = std::env::args().collect::<Vec<_>>();

    println!("Running the main Moxin binary:
        working directory: {}
        args: {:?}",
        current_exe_dir.display(),
        args,
    );

    // On Windows, the MOXIN_APP_BINARY needs to be able to find the WASMEDGE_MAIN_DYLIB (wasmedge.dll),
    // so we prepend it to the PATH.
    #[cfg(windows)] {
        match (std::env::var_os(ENV_PATH), _main_wasmedge_dylib_dir) {
            (Some(path), Some(dylib_parent)) => {
                println!("Prepending \"{}\" to Windows PATH", dylib_parent.display());
                let new_path = std::env::join_paths(
                    Some(dylib_parent.to_path_buf())
                        .into_iter()
                        .chain(std::env::split_paths(&path))
                )
                .expect("BUG: failed to join paths for the main Moxin binary.");
                std::env::set_var(ENV_PATH, &new_path);

            }
            _ => eprintln!("BUG: failed to set PATH for the main Moxin binary."),
        }
    }

    let _output = Command::new(current_exe_dir.join(MOXIN_APP_BINARY))
        .current_dir(current_exe_dir)
        .args(args.into_iter().skip(1)) // skip the first arg (the binary name)
        .spawn()?
        .wait_with_output()?;

    Ok(())
}


/// Checks that the current CPU supports AVX512, which is required by the current
/// builds of WasmEdge 0.14.0 on Windows.
///
/// Does nothing for other platforms.
fn check_cpu_features() {
    #[cfg(windows)] {
        if !is_x86_feature_detected!("avx512f") {
            eprintln!("Feature aes: {}", is_x86_feature_detected!("aes"));
            eprintln!("Feature pclmulqdq: {}", is_x86_feature_detected!("pclmulqdq"));
            eprintln!("Feature rdrand: {}", is_x86_feature_detected!("rdrand"));
            eprintln!("Feature rdseed: {}", is_x86_feature_detected!("rdseed"));
            eprintln!("Feature tsc: {}", is_x86_feature_detected!("tsc"));
            eprintln!("Feature mmx: {}", is_x86_feature_detected!("mmx"));
            eprintln!("Feature sse: {}", is_x86_feature_detected!("sse"));
            eprintln!("Feature sse2: {}", is_x86_feature_detected!("sse2"));
            eprintln!("Feature sse3: {}", is_x86_feature_detected!("sse3"));
            eprintln!("Feature ssse3: {}", is_x86_feature_detected!("ssse3"));
            eprintln!("Feature sse4.1: {}", is_x86_feature_detected!("sse4.1"));
            eprintln!("Feature sse4.2: {}", is_x86_feature_detected!("sse4.2"));
            eprintln!("Feature sse4a: {}", is_x86_feature_detected!("sse4a"));
            eprintln!("Feature sha: {}", is_x86_feature_detected!("sha"));
            eprintln!("Feature avx: {}", is_x86_feature_detected!("avx"));
            eprintln!("Feature avx2: {}", is_x86_feature_detected!("avx2"));
            eprintln!("Feature avx512f: {}", is_x86_feature_detected!("avx512f"));
            eprintln!("Feature avx512cd: {}", is_x86_feature_detected!("avx512cd"));
            eprintln!("Feature avx512er: {}", is_x86_feature_detected!("avx512er"));
            eprintln!("Feature avx512pf: {}", is_x86_feature_detected!("avx512pf"));
            eprintln!("Feature avx512bw: {}", is_x86_feature_detected!("avx512bw"));
            eprintln!("Feature avx512dq: {}", is_x86_feature_detected!("avx512dq"));
            eprintln!("Feature avx512vl: {}", is_x86_feature_detected!("avx512vl"));
            eprintln!("Feature avx512ifma: {}", is_x86_feature_detected!("avx512ifma"));
            eprintln!("Feature avx512vbmi: {}", is_x86_feature_detected!("avx512vbmi"));
            eprintln!("Feature avx512vpopcntdq: {}", is_x86_feature_detected!("avx512vpopcntdq"));
            eprintln!("Feature avx512vbmi2: {}", is_x86_feature_detected!("avx512vbmi2"));
            eprintln!("Feature gfni: {}", is_x86_feature_detected!("gfni"));
            eprintln!("Feature vaes: {}", is_x86_feature_detected!("vaes"));
            eprintln!("Feature vpclmulqdq: {}", is_x86_feature_detected!("vpclmulqdq"));
            eprintln!("Feature avx512vnni: {}", is_x86_feature_detected!("avx512vnni"));
            eprintln!("Feature avx512bitalg: {}", is_x86_feature_detected!("avx512bitalg"));
            eprintln!("Feature avx512bf16: {}", is_x86_feature_detected!("avx512bf16"));
            eprintln!("Feature avx512vp2intersect: {}", is_x86_feature_detected!("avx512vp2intersect"));
            // eprintln!("Feature avx512fp16: {}", is_x86_feature_detected!("avx512fp16"));
            eprintln!("Feature f16c: {}", is_x86_feature_detected!("f16c"));
            eprintln!("Feature fma: {}", is_x86_feature_detected!("fma"));
            eprintln!("Feature bmi1: {}", is_x86_feature_detected!("bmi1"));
            eprintln!("Feature bmi2: {}", is_x86_feature_detected!("bmi2"));
            eprintln!("Feature abm: {}", is_x86_feature_detected!("abm"));
            eprintln!("Feature lzcnt: {}", is_x86_feature_detected!("lzcnt"));
            eprintln!("Feature tbm: {}", is_x86_feature_detected!("tbm"));
            eprintln!("Feature popcnt: {}", is_x86_feature_detected!("popcnt"));
            eprintln!("Feature fxsr: {}", is_x86_feature_detected!("fxsr"));
            eprintln!("Feature xsave: {}", is_x86_feature_detected!("xsave"));
            eprintln!("Feature xsaveopt: {}", is_x86_feature_detected!("xsaveopt"));
            eprintln!("Feature xsaves: {}", is_x86_feature_detected!("xsaves"));
            eprintln!("Feature xsavec: {}", is_x86_feature_detected!("xsavec"));
            eprintln!("Feature cmpxchg16b: {}", is_x86_feature_detected!("cmpxchg16b"));
            eprintln!("Feature adx: {}", is_x86_feature_detected!("adx"));
            eprintln!("Feature rtm: {}", is_x86_feature_detected!("rtm"));
            eprintln!("Feature movbe: {}", is_x86_feature_detected!("movbe"));
            eprintln!("Feature ermsb: {}", is_x86_feature_detected!("ermsb"));

            use windows_sys::Win32::UI::WindowsAndMessaging::{
                MessageBoxW, MB_ICONERROR, MB_SETFOREGROUND, MB_TOPMOST,
            };
            // SAFE: just displaying an Error dialog box; the program will be terminated regardless.
            unsafe {
                MessageBoxW(
                    0,
                    windows_sys::w!(
                        "This CPU does not support AVX512, which is required by Moxin.\n\n\
                        The list of supported CPU features has been logged to the console.\
                    "),
                    windows_sys::w!("Error: Unsupported CPU!"),
                    MB_SETFOREGROUND | MB_TOPMOST | MB_ICONERROR,
                );
            }
            panic!("\nError: this CPU does not support AVX512, which is required by Moxin.\n")
        }
    }
}
