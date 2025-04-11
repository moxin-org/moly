# Moly: a Rust AI LLM client built atop [Robius](https://github.com/project-robius)

Moly is an AI LLM client written in Rust, that demonstrates the power of the [Makepad UI toolkit](https://github.com/makepad/makepad) and [Project Robius](https://github.com/project-robius), a framework for multi-platform application development in Rust.

> ⚠️ Moly is in early development. Please [file an issue](https://github.com/moxin-org/moly/issues/new) if you encounter bugs or unexpected results.

## AI Providers

The Moly app supports different types of AI providers:
1. **OpenAI-compatible AI providers**: configured through the Providers Dashboard.
   - Support for other clients will be added to MolyKit. To create your own custom clients, checkout the MolyKit documentation.
   - If you want to contribute providers, or extend the list of supported models for a given provider, see [instructions here](#contributing)
2. **Moly Server**: a local LLM backend that allows exploring, downloading and running OSS LLMs locally. For usage and installation see [instructions here](#running-moly-with-moly-server)
3. **MoFa Servers**: MoFa is a framework for building AI agents. Using MoFa, AI agents can be constructed via templates, and then exposed via a Dora server that is OpenAI-compatible. MoFa servers can be added to the application through the Providers Dashboard. See [instructions here](#running-moly-with-mofa).

## Building and Running

The following table shows which host systems can currently be used to build Moly for which target platforms.

<!-- prettier-ignore-start -->
| Host OS | Target Platform | Builds? | Runs? |
| ------- | --------------- | ------- | ----- | 
| macOS   | macOS           | ✅      | ✅    |
| Linux   | Linux           | ✅      | ✅    |
| Windows | Windows (10+)   | ✅      | ✅    |
<!-- prettier-ignore-end -->

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

> [!IMPORTANT]
> If your CPU does not support AVX512, then you should append the `--noavx` option onto the above command.
> If you use [`moly-runner`](#tip-use-moly-runner-for-easy-setup), it will handle this for you.

---

## Running Moly with Moly Server
[Moly Server](https://github.com/moxin-org/moly-server) is a local HTTP server which provides capabilities for searching, downloading, and running local LLMs over an OpenAI-compatible API. While not required for use of Moly, it can be run alongside and connected to by the main Moly application.
After following the [setup guide](https://github.com/moxin-org/moly-server?tab=readme-ov-file#building-and-running) in its README, the server can be run with:
```bash
cd moly-server/
cargo run -p moly-server
```
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