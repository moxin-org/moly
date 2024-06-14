# Moxin: a Rust AI LLM client built atop [Robius](https://github.com/project-robius)

Moxin is an AI LLM client written in Rust to demonstrate the functionality of the Robius, a framework for multi-platform application development in Rust.

> ⚠️ Moxin is just getting started and is not yet fully functional.

The following table shows which host systems can currently be used to build Moxin for which target platforms.
| Host OS | Target Platform | Builds? | Runs? |
| ------- | --------------- | ------- | ----- |
| macOS   | macOS           | ✅      | ✅    |  
| Linux   | ubuntu(x86_64-unknown-linux-gnu) | ✅ | ? |

## Building and Running

First, [install Rust](https://www.rust-lang.org/tools/install).

Then, install the required WasmEdge WASM runtime:

```sh
curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash

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

Alternatively, on the Linux platform, you need to ensure the following dependencies are installed (e.g., Ubuntu.):

```sh
sudo apt-get update
# openssl
sudo apt-get install libssl-dev pkg-config
# libclang for bindgen
sudo apt-get install llvm clang libclang-dev
# binfmt
sudo apt install binfmt-support
# Xcursor、X11、asound and pulse
sudo apt-get install libxcursor-dev libx11-dev libasound2-dev libpulse-dev
```

Then, run:

```sh
cd moxin
cargo run
```


## Packaging Moxin for Distribution

Install cargo packager:
```sh
cargo install --locked cargo-packager
```

### Packaging for macOS
Use `cargo packager` to generate a `.app` bundle and a `.dmg` disk image:
```sh
cargo packager --release --verbose   ## --verbose is optional
```

If you receive the following error:
```
ERROR cargo_packager::cli: Error running create-dmg script: File exists (os error 17)
```
then open Finder and unmount any Moxin-related disk images, then try the above `cargo packager` command again.

At this point, in the `dist/` directory you should see both the `Moxin.app` and the `.dmg`.


If you'd like to modify the .dmg background, here is the [Google Drawings file used to generate the MacOS .dmg background image](https://docs.google.com/drawings/d/1Uq13nAsCKFrl4s16HeLqpVfQ-vbF7v2Z8HFyqgeyrbE/edit?usp=sharing).
