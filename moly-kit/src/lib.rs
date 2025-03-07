//! # Basic usage
//!
//! ## Installation
//! Add MolyKit to your dependencies.
//! ```toml
//! moly-kit = { version = "*", features = ["full"] }
//! ```
//!
//! ## DSL
//! Import the batteries-included [Chat] widget.
//! ```
//! use moly_kit::widgets::chat::Chat;
//! ```
//! Then, add it somewhere in your parent widget.
//! ```
//! chat = <Chat> {}
//! ```
//!
//! ## Rust
//! We need to give the chat a [BotRepo], which allows [Chat] to pull information
//! about the available bots and send messages to them.
//!
//! A [BotRepo] can be directly created from a [BotClient], which interacts
//! with (mostly remote) bot providers like OpenAI, Ollama, OpenRouter,
//! Moly Server, MoFa, etc.
//!
//! An OpenAI compatible client comes out-of-the-box with MolyKit, [MolyClient].
//!
//! So, add the following somewhere appropriate (like in `after_new_from_doc`
//! from Makepad) to give [Chat] its [BotRepo]:
//! ```rust
//! use moly_kit::*
//!
//! let mut client = MolyClient::new("https://api.openai.com".into());
//! client.set_key("<YOUR_KEY>".into());
//!
//! let repo = BotRepo::from(client);
//!
//! self.chat(id!(chat)).write_with(|chat| {
//!   chat.bot_repo = repo;
//! })
//! ```
//!
//! # Advanced usage
//! [Chat] is by default a very automatic widget, but it exposes its lifecycle
//! through a "hook" mechanism. This mechanism allows you to:
//! - Know that something will happen.
//! - Know that something already happened.
//! - Abort something before it happens.
//! - Replace that aborted thing with something else.
//! - Inject tasks to be executed by the [Chat].
//! - Change the payload of a task affecting the operation when executed.
//!
//! TODO: Continue writing and give examples.
//!
//! # Web support
//!
//! If you want to use the built-in web support (the `async-web` flag), you may need
//! to do the following at `main.rs`:
//!
//! ```rust
//! #[cfg(target_arch = "wasm32")]
//! use wasm_bindgen::prelude::*;
//!
//! #[cfg(target_arch = "wasm32")]
//! fn main() {
//!     your_application_lib::app::app_main()
//! }
//!
//! #[cfg(not(target_arch = "wasm32"))]
//! #[tokio::main]
//! async fn main() {
//!     your_application_lib::app::app_main()
//! }
//! ```
//!
//! And then run your app with `wasm-bindgen` support like this:
//!
//! ```sh
//! cargo makepad wasm --bindgen run -p your_application_package
//! ```

pub mod clients;
pub mod protocol;
pub mod utils;
pub mod widgets;

pub use clients::*;
pub use protocol::*;
pub use widgets::*;
