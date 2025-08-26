![Moly Logo](https://github.com/user-attachments/assets/a899218e-5ef5-46df-bf36-d220edd89d2d)

# Moly: a Rust AI LLM client built atop [Robius](https://github.com/project-robius)

Moly is an AI LLM client written in Rust, that demonstrates the power of the [Makepad UI toolkit](https://github.com/makepad/makepad) and [Project Robius](https://github.com/project-robius), a framework for multi-platform application development in Rust.

> ⚠️ Moly is in early development. Please [file an issue](https://github.com/moxin-org/moly/issues/new) if you encounter bugs or unexpected results.

https://github.com/user-attachments/assets/bc50f75d-c82a-49c4-8faa-363afff198a1

### Download Pre-built Releases

Want to try Moly without building it from source? You can download the latest stable [pre-built releases of Moly](https://github.com/moxin-org/moly/releases).

The following table shows which host systems can currently be used to build Moly for which target platforms.

<!-- prettier-ignore-start -->
| Host OS | Target Platform | Builds? | Runs? | Packaging Support                            |
| ------- | --------------- | ------- | ----- | -------------------------------------------- |
| macOS   | macOS           | ✅       | ✅     | `.app`, [`.dmg`]                             |
| Linux   | Linux           | ✅       | ✅     | [`.deb` (Debian dpkg)], [AppImage], [pacman] |
| Windows | Windows (10+)   | ✅       | ✅     | `.exe` (NSIS)                                |
| Any     | Web             | ✅       | ✅     | N/A                                          |
| Any     | Android         | ✅       | ✅     | TODO                                          |
| macOS   | iOS             | ✅       | ✅     | TODO                                          |

<!-- prettier-ignore-end -->

## Features

### AI Providers

![Screenshot 2025-05-14 at 11 38 52 AM](https://github.com/user-attachments/assets/7d1ddbff-2872-43fd-8408-1624d8743bd1)

The Moly app supports different types of AI providers:
1. **OpenAI-compatible AI providers**: configured through the Providers Dashboard.
   - Support for other clients will be added to MolyKit. To create your own custom clients, checkout the [MolyKit documentation](https://moxin-org.github.io/moly/).
   - If you want to contribute providers, or extend the list of supported models for a given provider, see [instructions here](#contributing)
2. **Moly Server**: a local LLM backend that allows exploring, downloading and running OSS LLMs locally. For usage and installation see [instructions here](#running-moly-with-moly-server)
3. **MoFa Servers**: MoFa is a framework for building AI agents. Using MoFa, AI agents can be constructed via templates, and then exposed via a Dora server that is OpenAI-compatible. MoFa servers can be added to the application through the Providers Dashboard. See [instructions here](#running-moly-with-mofa).

### Local LLMS via Moly Server

[Moly Server](https://github.com/moxin-org/moly-server) is a local HTTP server which provides capabilities for searching, downloading, and running local LLMs over an OpenAI-compatible API.
While not required in order to use Moly, it can be run alongside the main Moly application for an integrated, local experience.

![Screenshot 2025-05-14 at 11 40 43 AM](https://github.com/user-attachments/assets/1235cc98-a175-4a8f-89a9-4789d4716189)


![Screenshot 2025-05-14 at 11 41 21 AM](https://github.com/user-attachments/assets/22570566-6726-488d-8b92-2282e6be78e8)


To get started, simply download and extract the latest version for your platform from the [server releases page](https://github.com/moxin-org/moly-server/releases) and run the executable in a command line from inside the directory.

Alternatively, to compile it from source, follow the [setup guide](https://github.com/moxin-org/moly-server?tab=readme-ov-file#building-and-running) and then run:
```bash
cd moly-server/
cargo run -p moly-server
```

## Building and Running (native)

1. [Install Rust](https://www.rust-lang.org/tools/install).

2. Obtain the source code for this repository:

```sh
git clone https://github.com/moxin-org/moly.git
```

3. Run
```sh
cargo run --release
```

### Linux requirements
> [!IMPORTANT]
> If your CPU does not support AVX512, then you should append the `--noavx` option onto the above command.
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

## Building and Running (web)

1. Install [Rust](https://www.rust-lang.org/tools/install) and [cargo-makepad](https://github.com/makepad/makepad/tree/dev).
2. Obtain the source code for this repository:

```sh
git clone https://github.com/moxin-org/moly.git
```

3. Run and serve the Moly app:

```sh
cargo makepad wasm --bindgen run -p moly --release
```

> [!NOTE]
> If you want to deploy it, it's recommended to optimize for size building it
> like this:
>
> ```sh
> cargo makepad wasm --strip --brotli --bindgen build -p moly --profile=small
> ```


### Packaging Moly for Distribution

> Note: we already have [pre-built releases of Moly](https://github.com/moxin-org/moly/releases) available for download.

Install `cargo-packager`:

```sh
rustup update stable  ## Rust version 1.79 or higher is required
cargo +stable install --force --locked cargo-packager
```

For posterity, these instructions have been tested on `cargo-packager` version 0.10.1, which requires Rust v1.79.

#### Packaging for Linux

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
sudo apt install ./moly_*.deb  ## Replace * with version/arch. The leading "./" part is required
```

We recommend using `apt install` to install the `.deb` file instead of `dpkg -i`, because `apt` will auto-install all of Moly's required dependencies, whereas `dpkg` will require you to install them manually.

To run the AppImage bundle, simply set the file as executable and then run it:

```sh
cd dist/
chmod +x moly_*.AppImage ## Replace * with version/arch
./moly_*.AppImage ## Replace * with version/arch
```

#### Packaging for Windows

This can only be run on an actual Windows machine, due to platform restrictions.

First, install the necessary build tools if you haven't already (e.g., Visual Studio Build Tools, LLVM as mentioned in some setups).

Ensure you are in the root `moly` directory, and then you can use `cargo packager` to generate a `setup.exe` file using NSIS:

```sh
cargo packager --release --formats nsis --verbose  ## --verbose is optional
```

After the command completes, you should see a Windows installer called `moly_*_x64-setup.exe` (replace * with version) in the `dist/` directory.
Double-click that file to install Moly on your machine, and then run it as you would a regular application.

#### Packaging for macOS

This can only be run on an actual macOS machine, due to platform restrictions.

Ensure you are in the root `moly` directory, and then you can use `cargo packager` to generate an `.app` bundle and a `.dmg` disk image:

```sh
cargo packager --release --verbose  ## --verbose is optional
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
You can immediately double-click the `Moly.app` bundle to run it, or you can double-click the `.dmg` file to install.

> Note that the `.dmg` is what should be distributed for installation on other machines, not the `.app`.

If you'd like to modify the .dmg background, here is the [Google Drawings file used to generate the MacOS .dmg background image](https://docs.google.com/drawings/d/1Uq13nAsCKFrl4s16HeLqpVfQ-vbF7v2Z8HFyqgeyrbE/edit?usp=sharing).

[`.dmg`]: https://support.apple.com/en-gb/guide/mac-help/mh35835/mac
[`.deb` (Debian dpkg)]: https://www.debian.org/doc/manuals/debian-faq/pkg-basics.en.html#package
[AppImage]: https://appimage.org/
[pacman]: https://pacman.archlinux.page/pacman.8.html

---

## Running Moly with MoFa

[MoFa](https://github.com/moxin-org/mofa) is a software framework for building AI agents. Moly supports connecting to MoFa servers to interact with AI agents in the same way it does with local or remote LLMs.

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

Navigate to the folder of the Dora node that implements the http server 
```bash
cd examples/moly_client
```
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

### Connect Moly to MoFa

Go to the Providers Dashboard and enable the MoFa entry (or add new ones if needed)

## Contributing

### Extending the default supported providers

One of the easiest ways to contribute to Moly is by extending the list of predefined supported providers and their models.

#### How to add a new provider:
1.	Add the provider information to [supported_providers.json](src/data/supported_providers.json).
   - `name`: The name to display in the UI
   - `url`: The full API endpoint for this provider, including versioning, e.g. "https://api.openai.com/v1"
   - `provider_type`: The type of API format that the provider uses, e.g. the `"provider_type": "OpenAI"` will use the `OpenAIClient` from MolyKit. In Moly, the mapping between supported provider types and MolyKit clients can be found in [src/chat/chat_screen.rs](src/chat/chat_screen.rs) (if you were to add a custom MolyKit client and default supported provider, you would need to extend the mapping here).
   - `supported_models`: A list of model ids to be used as the whitelist of allowed/supported models in Moly for this provider.

2.	Add a new icon for the provider under [/resources/images/providers](/resources/images/providers) (in PNG format), using the **same name** as the provider you registered in the previous step.

3.	Update the providers view, importing the new image and referencing the import:
   - At the top of the live_design!{} block, add your import, e.g.:
   ```rs
   ICON_GEMINI = dep("crate://self/resources/images/providers/gemini.png")
   ```
   - Add the icon to the list of provider_icons:
   ```rs
   provider_icons: [
      ...
      (ICON_GEMINI), // Add this line to reference the imported file.
   ]
   ```
