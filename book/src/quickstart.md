# Quickstart
## Prerequisites

This guide assumes you are familiar with Makepad and you have a
bare-bones app ready to start integrating Moly Kit while following this guide.

## Installation

Add Moly Kit to your `Cargo.toml` dependencies:

```toml
moly-kit = { git = "https://github.com/moxin-org/moly.git", features = ["full"], branch = "main" }
```

> **Tip:** Change `branch = "main"` to (for example) `tag = "v0.2.1"` if you want to
> stay on a stable version.

If you are targeting native (non-web) platforms, you will also need to add `tokio`
to your app. Even if you don't use it directly, Moly Kit will.

```shell
cargo add tokio -F full
```

```rust
#[tokio::main]
async fn main() {
    your_amazing_app::app::app_main()
}
```

> **Note:** `tokio` is not needed if you are only targeting web platforms. More details
> about targeting web will be covered in the [Web support](web.md) guide.

## Register widgets

As with any Makepad app, we need to register the widgets we want to use in the `live_register`
of your app before any widget that uses Moly Kit.

```rust
impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
        
        // Add this line
        moly_kit::live_design(cx);

        crate::your_amazing_widgets::live_design(cx);
    }
}
```

## DSL

Import the batteries-included `Chat` widget into your own widget and place it
somewhere.

```rust
live_design! {
    use link::theme::*;
    use link::widgets::*;

    // Add this line
    use moly_kit::widgets::chat::Chat;
    

    pub YourAmazingWidget = {{YourAmazingWidget}} {
        // And this line
        chat = <Chat> {}
    }
}
```

## Rust-side configuration

The `Chat` widget as it is will not work. We need to configure some one-time stuff
from the Rust side.

The `Chat` widget pulls information about available bots from a synchronous interface
called a `BotContext`. We don't need to understand how it works, but we need to create
and pass one to `Chat`.

A `BotContext` can be directly created from a `BotClient`, which is an asynchronous
interface to interact with (mostly remote) bot providers like OpenAI, Ollama, OpenRouter,
Moly Server, MoFa, etc.

Once again, we don't need to understand how a `BotClient` works (unless you need
to implement your own) as Moly Kit already comes with some built-in ones. We can
simply use `OpenAIClient` to interact with any OpenAI-compatible remote API.

We should ensure this configuration code runs once and before the `Chat` widget
is used by Makepad, so a good place to write it is in Makepad's `after_new_from_doc`
lifecycle hook. The practical tl;dr of all this theory would be simply the following:

```rust
use moly_kit::*;

impl LiveHook for YourAmazingWidget {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        let mut client = OpenAIClient::new("https://api.openai.com/v1".into());
        client.set_key("<YOUR_KEY>".into());

        let context = BotContext::from(client);

        let mut chat = self.chat(id!(chat));
        chat.write().bot_context = context;
    }
}
```

> **Note:** Moly Kit doesn't duplicate methods from `Chat` into Makepad's autogenerated
> `ChatRef` but provides `read()` and `write()` helpers to access the inner widget.