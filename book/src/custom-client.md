# Implement your own client
## Prerequisites
This guide assumes you have already read the [Quickstart](quickstart.md).

## Introduction

As we mentioned before, a "client" is something that allows us to interact with
models/agents from providers like OpenAI, Ollama, etc., asynchronously.

In general, as long as Moly Kit has a client compatible with the provider you want
to connect to, you don't need to implement your own client. For example, you may
recall from the [Quickstart](quickstart.md) that there is a built-in `OpenAIClient`
that you can use with any OpenAI-compatible API.

You also have the `MultiClient` we mentioned in [Use multiple providers](multiple-providers.md),
which is a utility client to compose multiple clients into one.

All these clients are clients because they implement the `BotClient` trait. This
trait defines a set of (mostly) asynchronous methods to fetch and send data to providers.

If we want to build our own client, either to support an unsupported provider,
to make a utility, or for any other reason, you simply need to implement that trait.

## Implementing the `BotClient` trait

Some of the methods in this trait have default implementations, so we don't need
to implement them, and they may not be relevant for us.

The ones that are very important to implement are the following:

```rust
struct MyCustomClient {
  // ... some config fields ...
}


impl BotClient for MyCustomClient {
    fn bots(&self) -> BoxPlatformSendFuture<'static, ClientResult<Vec<Bot>>> {
        let future = async move {
          // ... fetch our list of available models/agents ...
        };

        Box::pin(future)
    }

    fn send(
        &mut self,
        bot_id: &BotId,
        messages: &[Message],
        tools: &[Tool],
    ) -> BoxPlatformSendStream<'static, ClientResult<MessageContent>> {
        let stream = stream! {
          // ... write some code that yields chunks of the message content ...
        };

        Box::pin(stream)
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(MyCustomClient)
    }
}
```

From this implementation template, we can see a lot of details. First of all, the async
methods return `BoxPlatformSendFuture` and `BoxPlatformSendStream` respectively.
These are basically boxed dynamic futures/streams with some cross-platform considerations.

```admonish warning
Boxed dynamic futures/streams are one of the things that can make async
code in Rust difficult to write. This is because this kind of box erases important
type information that Rust would normally use to make our lives 10 times easier.

However, since generics will not play well with Makepad widgets when they reach the
UI, dynamic dispatching is a necessary evil.
```

We can also see these methods normally return a `ClientResult`, which is slightly different
from Rust's standard `Result` as it can contain both successfully recovered data and multiple
errors. However, the constructor methods of `ClientResult` enforce semantics
similar to `Result`. If you use `new_ok(...)`, it will contain just a success value. If you use
`new_err(...)`, it will contain just an error. And if you use `new_ok_and_err`, you are expected
to pass a non-empty list of errors alongside your success data; otherwise, a default error will
be inserted into the list, and you probably don't want that.

Let's talk a little more about what the methods do. The `bots` method simply returns
a list of available models/agents. It's pretty simple to implement; you could use
something like `reqwest` to fetch some JSON from your provider with the list of models,
parse that, and return it.

`send` is the method that sends a message to a model/agent. It returns a
stream, where each item yielded will be a full snapshot of the message content as
it is being streamed. In practice, this means the last item of the stream is the
final message content.

```admonish tip
Different from `Future`s in Rust, `Stream`s are a little less mature, so creating them
with the `async_stream` crate is advised for simplicity.
```

Note that the exact implementation of a client greatly depends on the provider you
are trying to support, so it's difficult to make a generic guide on how to build it
step by step. I recommend checking how `OpenAIClient` and `MultiClient` are implemented
as examples to create your own.

But to avoid leaving this section without a working client, let's finish the implementation
with some dummy methods for a client that simply repeats the last message.

```rust
struct EchoClient;

impl BotClient for EchoClient {
    fn bots(&self) -> BoxPlatformSendFuture<'static, ClientResult<Vec<Bot>>> {
        let future = futures::future::ready(ClientResult::new_ok(vec![Bot {
            id: BotId::new("echo-provider", "echo-echo"),
            name: "Echo Echo".to_string(),
            avatar: Picture::Grapheme("E".into()),
        }]));

        Box::pin(future)
    }

    fn send(
        &mut self,
        bot_id: &BotId,
        messages: &[Message],
        tools: &[Tool],
    ) -> BoxPlatformSendStream<'static, ClientResult<MessageContent>> {
        let last = messages.last().map(|m| m.content.text.clone()).unwrap_or_default();

        let stream = futures::stream::once(async move {
            ClientResult::new_ok(MessageContent {
                text: format!("Echo: {}", last),
                ..Default::default()
            })
        });

        Box::pin(stream)
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(EchoClient)
    }
}
```

```admonish warning
Try not to use `tokio`-specific code

Most relevant Tokio utilities are present in the `futures` crate, which is the base
for most async crates out there (including Tokio).

Using Tokio inside your client implementation would make it unusable on the web.

Of course, if your custom client deals with native-specific stuff like the filesystem,
stdio, etc., then it may be reasonable.
```
