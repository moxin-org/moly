# Moly: a Rust AI LLM client built atop [Robius](https://github.com/project-robius)

Moly is an AI LLM client written in Rust, and demonstrates the power of the [Makepad UI toolkit](https://github.com/makepad/makepad) and [Project Robius](https://github.com/project-robius), a framework for multi-platform application development in Rust.

> ⚠️ Moly is in beta. Please [file an issue](https://github.com/moxin-org/moly/issues/new) if you encounter bugs or unexpected results.

The following table shows which host systems can currently be used to build Moly for which target platforms.

<!-- prettier-ignore-start -->
| Host OS | Target Platform | Builds? | Runs? | Packaging Support                            |
| ------- | --------------- | ------- | ----- | -------------------------------------------- |
| macOS   | macOS           | ✅      | ✅    | `.app`, [`.dmg`]                             |
| Linux   | Linux           | ✅      | ✅    | [`.deb` (Debian dpkg)], [AppImage], [pacman] |
| Windows | Windows (10+)   | ✅      | ✅    | `.exe` (NSIS)                                |
<!-- prettier-ignore-end -->

## Building and Running

1. [Install Rust](https://www.rust-lang.org/tools/install).

2. Obtain the source code for this repository:

```sh
git clone https://github.com/moxin-org/moly.git
```

#### Tip: use `moly-runner` for easy setup

> [!TIP]
> On all platforms, you can use our helper program to auto-setup WasmEdge for you and run any `cargo` command:
>
> ```sh
> cargo run -p moly-runner -- --install    ## finds or installs WasmEdge, then stops.
> cargo run -p moly-runner -- cargo build  ## builds Moly
> cargo run -p moly-runner -- cargo run    ## builds and runs Moly
> cargo run -p moly-runner -- cargo [your-command-here]
> ```

### macOS

Install the required WasmEdge WASM runtime (or use [`moly-runner`](#tip-use-moly-runner-for-easy-setup)):

```sh
curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash -s -- --version=0.14.1

source $HOME/.wasmedge/env
```

Then use `cargo` to build and run Moly:

```sh
cd moly
cargo run --release
```

### Linux

Install the required WasmEdge WASM runtime (or use [`moly-runner`](#tip-use-moly-runner-for-easy-setup)):

```sh
curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash -s -- --version=0.14.1

source $HOME/.wasmedge/env
```

> [!IMPORTANT]
> If your CPU does not support AVX512, then you should append the `--noavx` option onto the above command.
> If you use [`moly-runner`](#tip-use-moly-runner-for-easy-setup), it will handle this for you.

To build Moly on Linux, you must install the following dependencies:
`openssl`, `clang`/`libclang`, `binfmt`, `Xcursor`/`X11`, `asound`/`pulse`.
On a Debian-like Linux distro (e.g., Ubuntu), run the following:

```sh
sudo apt-get update
sudo apt-get install libssl-dev pkg-config llvm clang libclang-dev binfmt-support libxcursor-dev libx11-dev libasound2-dev libpulse-dev
```

Then use `cargo` to build and run Moly:

```sh
cd moly
cargo run --release
```

## Windows (Windows 10, Windows 11 or higher)

1.  Download and install the LLVM v17.0.6 release for Windows: [Here is a direct link to LLVM-17.0.6-win64.exe](https://github.com/llvm/llvm-project/releases/download/llvmorg-17.0.6/LLVM-17.0.6-win64.exe), 333MB in size.

> [!IMPORTANT]
> During the setup procedure, make sure to select `Add LLVM to the system PATH for all users` or `for the current user`.

2. Restart your PC, or log out and log back in, which allows the LLVM path to be properly

   - Alternatively you can add the LLVM path `C:\Program Files\LLVM\bin` to your system PATH.

3. Use `moly-runner` to auto-setup WasmEdge and then build & run Moly:

```sh
cargo run -p moly-runner -- cargo run --release  ## `--release` is optional
```

---

## Packaging Moly for Distribution

> Note: we already have [pre-built releases of Moly](https://github.com/moxin-org/moly/releases) available for download.

Install `cargo-packager`:

```sh
rustup update stable  ## Rust version 1.79 or higher is required
cargo +stable install --force --locked cargo-packager
```

For posterity, these instructions have been tested on `cargo-packager` version 0.10.1, which requires Rust v1.79.

### Packaging for Linux

On a Debian-based Linux distribution (e.g., Ubuntu), you can generate a `.deb` Debian package, an AppImage, and a pacman installation package.

> [!IMPORTANT]
> You can only generate a `.deb` Debian package on a Debian-based Linux distribution, as `dpkg` is needed.

> [!NOTE]
> The `pacman` package has not yet been tested.

Ensure you are in the root `moly` directory, and then you can use `cargo packager` to generate all three package types at once:

```sh
cargo packager --release --verbose   ## --verbose is optional
```

To install the Moly app from the `.deb`package on a Debian-based Linux distribution (e.g., Ubuntu), run:

```sh
cd dist/
sudo apt install ./moly_0.1.0_amd64.deb  ## The "./" part is required
```

We recommend using `apt install` to install the `.deb` file instead of `dpkg -i`, because `apt` will auto-install all of Moly's required dependencies, whereas `dpkg` will require you to install them manually.

To run the AppImage bundle, simply set the file as executable and then run it:

```sh
cd dist/
chmod +x moly_0.1.0_x86_64.AppImage
./moly_0.1.0_x86_64.AppImage
```

### Packaging for Windows

This can only be run on an actual Windows machine, due to platform restrictions.

First, [follow the above instructions for building on Windows](#windows-windows-10-windows-11-or-higher).

Ensure you are in the root `moly` directory, and then you can use `cargo packager` to generate a `setup.exe` file using NSIS:

```sh
cargo run -p moly-runner -- cargo packager --release --formats nsis --verbose   ## --verbose is optional
```

After the command completes, you should see a Windows installer called `moly_0.1.0_x64-setup` in the `dist/` directory.
Double-click that file to install Moly on your machine, and then run it as you would a regular application.

### Packaging for macOS

This can only be run on an actual macOS machine, due to platform restrictions.

Ensure you are in the root `moly` directory, and then you can use `cargo packager` to generate an `.app` bundle and a `.dmg` disk image:

```sh
cargo packager --release --verbose   ## --verbose is optional
```

> [!IMPORTANT]
> You will see a .dmg window pop up — please leave it alone, it will auto-close once the packaging procedure has completed.

> [!TIP]
> If you receive the following error:
>
> ```
> ERROR cargo_packager::cli: Error running create-dmg script: File exists (os error 17)
> ```
>
> then open Finder and unmount any Moly-related disk images, then try the above `cargo packager` command again.

> [!TIP]
> If you receive an error like so:
>
> ```
> Creating disk image...
> hdiutil: create failed - Operation not permitted
> could not access /Volumes/Moly/Moly.app - Operation not permitted
> ```
>
> then you need to grant "App Management" permissions to the app in which you ran the `cargo packager` command, e.g., Terminal, Visual Studio Code, etc.
> To do this, open `System Preferences` → `Privacy & Security` → `App Management`,
> and then click the toggle switch next to the relevant app to enable that permission.
> Then, try the above `cargo packager` command again.

After the command completes, you should see both the `Moly.app` and the `.dmg` in the `dist/` directory.
You can immediately double-click the `Moly.app` bundle to run it, or you can double-click the `.dmg` file to

> Note that the `.dmg` is what should be distributed for installation on other machines, not the `.app`.

If you'd like to modify the .dmg background, here is the [Google Drawings file used to generate the MacOS .dmg background image](https://docs.google.com/drawings/d/1Uq13nAsCKFrl4s16HeLqpVfQ-vbF7v2Z8HFyqgeyrbE/edit?usp=sharing).

[`.dmg`]: https://support.apple.com/en-gb/guide/mac-help/mh35835/mac
[`.deb` (Debian dpkg)]: https://www.debian.org/doc/manuals/debian-faq/pkg-basics.en.html#package
[AppImage]: https://appimage.org/
[pacman]: https://pacman.archlinux.page/pacman.8.html

## Running Moly with Moly Server

[Moly Server](https://github.com/moxin-org/moly-server) is a local HTTP server which provides capabilities for searching, downloading, and running local LLMs over an OpenAI-compatible API. While not required for use of Moly, it can be run alongside and connected to by the main Moly application.

After following the [setup guide](https://github.com/moxin-org/moly-server?tab=readme-ov-file#building-and-running) in its README, the server can be run with:

```bash
cd moly-server/
cargo run -p moly-server
```

## Running Moly with MoFa

[MoFa](https://github.com/moxin-org/mofa) is a software framework for building AI agents. Moly supports connecting to MoFa servers to interact with AI agents in the same way it does with local LLMs.

To run Moly with a local MoFa server, you can follow these steps:

### 1. Install Dora

https://github.com/dora-rs/dora?tab=readme-ov-file#installation

### 2. Install MoFa

Requires python ^3.10

```bash
git clone https://github.com/moxin-org/mofa.git
```
Install the required Python libraries, and mainly,
the mofa library itself
```bash
cd python && pip install -r requirements.txt && pip install -e .
pip install dora-rs
```

### 3. Run the Moly client (MoFa server for Moly)

Folder of the Dora node that implements the http server
```bash
# 
cd examples/moly_client
Run MoFa with
```
dora up
dora build dataflow.yml
dora start dataflow.yml
```
If there's any error when doing dora start, you can restart dora
```bash
dora destroy && dora up
```

At this point the server should be up
You can verify it with a request for chat completion:
```bash
curl http://localhost:8000/v1/chat/completions \
-v -H "Content-Type: application/json" \
-d '{
"model": "moly-chat",
"messages": [
{ "role": "system", "content": "Use positive language and offer helpful solutions to their problems." },
{ "role": "user", "content": "What is the currency used in Spain?" }
],
"temperature": 0.7,
"stream": true
}'
```
This should return a JSON response with the completion.

## Connect Moly to MoFa

Go to Settings and make sure there's MoFa server listed with the URL [`http://localhost:8000`](http://localhost:8000/) (should be there by default).

> [!NOTE]
> For development, if you want to avoid running the MoFa server, you can fake it by setting the `MOFA_BACKEND` environment variable to `fake` (default is `real`):
> ```
> MOFA_BACKEND=fake cargo run
> ```
