# Moxin: a Rust AI LLM client built atop [Robius](https://github.com/project-robius)

Moxin is an AI LLM client written in Rust to demonstrate the functionality of the Robius, a framework for multi-platform application development in Rust.

> ⚠️ Moxin is just getting started and is not yet fully functional.

The following table shows which host systems can currently be used to build Moxin for which target platforms.
| Host OS | Target Platform | Builds? | Runs? | Packaging Support                            |
| ------- | --------------- | ------- | ----- | -------------------------------------------- |
| macOS   | macOS           | ✅      | ✅    | `.app`, [`.dmg`]                             |
| Linux   | Linux           | ✅      | ✅    | [`.deb` (Debian dpkg)], [AppImage], [pacman] |

## Building and Running

First, [install Rust](https://www.rust-lang.org/tools/install).

Then, install the required WasmEdge WASM runtime:

```sh
curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash -s -- --version=0.14.0

source $HOME/.wasmedge/env
```

Obtain the source code from this repository:
```sh
git clone https://github.com/moxin-org/moxin.git
```

### macOS

Then, on a standard desktop platform (macOS), simply run:

```sh
cd moxin
cargo run
```

### Linux

To build Moxin on Linux, you must install the following dependencies:
`openssl`, `clang`/`libclang`, `binfmt`, `Xcursor`/`X11`, `asound`/`pulse`.

On a Debian-like Linux distro (e.g., Ubuntu), run the following:
```sh
sudo apt-get update
sudo apt-get install libssl-dev pkg-config llvm clang libclang-dev binfmt-support libxcursor-dev libx11-dev libasound2-dev libpulse-dev
```

Then, run:

```sh
cd moxin
cargo run
```

## Windows (Windows 10 or higher)
1.  Download and install the LLVM v17.0.6 release for Windows: [Here is a direct link to LLVM-17.0.6-win64.exe](https://github.com/llvm/llvm-project/releases/download/llvmorg-17.0.6/LLVM-17.0.6-win64.exe), 333MB in size.

> [!IMPORTANT]
> During the setup procedure, make sure to select `Add LLVM to the system PATH for all users` or `for the current user`.

2. Restart your PC, or log out and log back in, which allows the LLVM path to be properly
    * Alternatively you can add the LLVM path `C:\Program Files\LLVM\bin` to your system PATH.
3.  Download the [WasmEdge-0.14.0-windows.zip](https://github.com/WasmEdge/WasmEdge/releases/download/0.14.0/WasmEdge-0.14.0-windows.zip) file from [the WasmEdge v0.14.0 release page](https://github.com/WasmEdge/WasmEdge/releases/tag/0.14.0),
    and then extract it into a directory of your choice.
    We recommend using your home directory (e.g., `C:\Users\<USERNAME>\`), represented by `$home` in powershell and `%homedrive%%homepath%` in batch-cmd.

    Afterwards, you should see a directory called `WasmEdge-0.14.0-Windows` there.
        
    To do this quickly in powershell:
    ```powershell
    $ProgressPreference = 'SilentlyContinue' ## makes downloads much faster
    Invoke-WebRequest -Uri "https://github.com/WasmEdge/WasmEdge/releases/download/0.14.0/WasmEdge-0.14.0-windows.zip" -OutFile "WasmEdge-0.14.0-windows.zip"
    Expand-Archive -Force -LiteralPath "WasmEdge-0.14.0-windows.zip" -DestinationPath $home
    $ProgressPreference = 'Continue' ## restore default progress bars
    ```

4. Download the WasmEdge WASI-NN plugin here: [WasmEdge-plugin-wasi_nn-ggml-0.14.0-windows_x86_64.zip](https://github.com/WasmEdge/WasmEdge/releases/download/0.14.0/WasmEdge-plugin-wasi_nn-ggml-0.14.0-windows_x86_64.zip) (15.5MB) and extract it to the same directory as above, e.g., `C:\Users\<USERNAME>\WasmEdge-0.14.0-Windows`.
> [!IMPORTANT]
> You will be asked whether you want to replace the files that already exist; select `Replace the files in the destination` when doing so.    
* To do this quickly in powershell:
    ```powershell
    $ProgressPreference = 'SilentlyContinue' ## makes downloads much faster
    Invoke-WebRequest -Uri "https://github.com/WasmEdge/WasmEdge/releases/download/0.14.0/WasmEdge-plugin-wasi_nn-ggml-0.14.0-windows_x86_64.zip" -OutFile "WasmEdge-plugin-wasi_nn-ggml-0.14.0-windows_x86_64.zip"
    Expand-Archive -Force -LiteralPath "WasmEdge-plugin-wasi_nn-ggml-0.14.0-windows_x86_64.zip" -DestinationPath "$home\WasmEdge-0.14.0-Windows"
    $ProgressPreference = 'Continue' ## restore default progress bars
    ```
    
5. Set the `WASMEDGE_DIR` and `WASMEDGE_PLUGIN_PATH` environment variables to point to the `WasmEdge-0.14.0-Windows` directory that you extracted above, and then build Moxin.
    In powershell, you can do this like so:
    ```powershell
    $env:WASMEDGE_DIR="$home\WasmEdge-0.14.0-Windows\"
    $env:WASMEDGE_PLUGIN_PATH="$home\WasmEdge-0.14.0-Windows\"
    cargo run
    ```

    In Windows `cmd`, you can do this like so:
    ```batch
    set WASMEDGE_DIR=%homedrive%%homepath%\WasmEdge-0.14.0-Windows
    set WASMEDGE_PLUGIN_PATH=%homedrive%%homepath%\WasmEdge-0.14.0-Windows
    cargo run
    ```

    In a Unix-like shell on Windows (e.g., GitBash, cygwin, msys2, WSL/WSL2):
    ```sh
    WASMEDGE_DIR=$HOME/WasmEdge-0.14.0-Windows \
    WASMEDGE_PLUGIN_PATH=$HOME/WasmEdge-0.14.0-Windows \
    cargo run
    ```


## Packaging Moxin for Distribution

Install `cargo-packager`:
```sh
cargo install --force --locked cargo-packager
```
For posterity, these instructions have been tested on `cargo-packager` version 0.10.1, which requires Rust v1.79.


### Packaging for Linux
On a Debian-based Linux distribution (e.g., Ubuntu), you can generate a `.deb` Debian package, an AppImage, and a pacman installation package.


> [!IMPORTANT]
> You can only generate a `.deb` Debian package on a Debian-based Linux distribution, as `dpkg` is needed.
 
> [!NOTE]
> The `pacman` package has not yet been tested.

Ensure you are in the root `moxin` directory, and then you can use `cargo packager` to generate all three package types at once:
```sh
cargo packager --release --verbose   ## --verbose is optional
```

To install the Moxin app from the `.deb`package on a Debian-based Linux distribution (e.g., Ubuntu), run:
```sh
cd dist/
sudo apt install ./moxin_0.1.0_amd64.deb  ## The "./" part is required
```
We recommend using `apt install` to install the `.deb` file instead of `dpkg -i`, because `apt` will auto-install all of Moxin's required dependencies, whereas `dpkg` will require you to install them manually.


To run the AppImage bundle, simply set the file as executable and then run it:
```sh
cd dist/
chmod +x moxin_0.1.0_x86_64.AppImage
./moxin_0.1.0_x86_64.AppImage
```

### Packaging for Windows
This can only be run on an actual Windows machine, due to platform restrictions.

First, [follow the above instructions for building on Windows](#windows-windows-10-or-higher).

Ensure you are in the root `moxin` directory, and then you can use `cargo packager` to generate a `setup.exe` file using NSIS:
```sh
WASMEDGE_DIR=path/to/WasmEdge-0.14.0-Windows cargo packager --release --formats nsis --verbose   ## --verbose is optional
```

After the command completes, you should see a Windows installer called `moxin_0.1.0_x64-setup` in the `dist/` directory.
Double-click that file to install Moxin on your machine, and then run it as you would a regular application.


### Packaging for macOS
This can only be run on an actual macOS machine, due to platform restrictions.

Ensure you are in the root `moxin` directory, and then you can use `cargo packager` to generate an `.app` bundle and a `.dmg` disk image:
```sh
cargo packager --release --verbose   ## --verbose is optional
```

> [!IMPORTANT]
> You will see a .dmg window pop up — please leave it alone, it will auto-close once the packaging procedure has completed.

> [!TIP]
> If you receive the following error:
> ```
> ERROR cargo_packager::cli: Error running create-dmg script: File exists (os error 17)
> ```
> then open Finder and unmount any Moxin-related disk images, then try the above `cargo packager` command again.

> [!TIP]
> If you receive an error like so:
> ```
> Creating disk image...
> hdiutil: create failed - Operation not permitted
> could not access /Volumes/Moxin/Moxin.app - Operation not permitted
> ```
> then you need to grant "App Management" permissions to the app in which you ran the `cargo packager` command, e.g., Terminal, Visual Studio Code, etc.
> To do this, open `System Preferences` → `Privacy & Security` → `App Management`,
> and then click the toggle switch next to the relevant app to enable that permission. 
> Then, try the above `cargo packager` command again.

After the command completes, you should see both the `Moxin.app` and the `.dmg` in the `dist/` directory.
You can immediately double-click the `Moxin.app` bundle to run it, or you can double-click the `.dmg` file to 

> Note that the `.dmg` is what should be distributed for installation on other machines, not the `.app`.

If you'd like to modify the .dmg background, here is the [Google Drawings file used to generate the MacOS .dmg background image](https://docs.google.com/drawings/d/1Uq13nAsCKFrl4s16HeLqpVfQ-vbF7v2Z8HFyqgeyrbE/edit?usp=sharing).


[`.dmg`]: https://support.apple.com/en-gb/guide/mac-help/mh35835/mac
[`.deb` (Debian dpkg)]: https://www.debian.org/doc/manuals/debian-faq/pkg-basics.en.html#package
[AppImage]: https://appimage.org/
[pacman]: https://pacman.archlinux.page/pacman.8.html
