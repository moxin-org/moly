# Implement your own client
## Prerequisites
This guide assumes you have already read the [Quickstart](quickstart.md).

## Introduction

As we mentioned before, a "client" is something that allows as to interact with
models/agents from providers like OpenAI, Ollama, etc., asynchronously.

In general, as long as Moly Kit have a client compatible with the provider you want
to connect to, you don't need to implement your own client. For example, you may
recall from the [Quickstart](quickstart.md) that there is a built-in `OpenAIClient`
that you can use with any OpenAI-compatible API.

You also have the `MultiClient` we mentioned on [Use multiple providers](multiple-providers.md)
which is an utility client to compose clients into one.

All this clients are clients because they implement the `BotClient` trait. This
trait defines a set of (mostly) asynchronous methods to fetch and send data to providers.

If we want to build ours own client, either to support some unsupported provider,
or to make an utility, or whatever, you simply need to implement that trait.

## Implementing the `BotClient` trait

Some of the methods in this trait have default implementations, so we donn't need
to implement them, and they may not be relevant for us.

The ones that are very important to implement are the following ones:

```rust
struct MyCustomClient {
  // ... some config fields ...
}


impl BotClient for MyCustomClient {
    fn bots(&self) -> MolyFuture<'static, ClientResult<Vec<Bot>>> {
        let future = async move {
          // ... fetch our list of available models/agenets ...
        };

        moly_future(future)
    }

    fn send_stream(
        &mut self,
        bot: &Bot,
        messages: &[Message],
    ) -> MolyStream<'static, ClientResult<MessageContent>> {
        let stream = stream! {
          // ... write some code that yields chunks of the message content ...
        };

        moly_stream(stream)
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(MyCustomClient)
    }
}
```

We can see from this implementation templates a lot of details. First of all, the async
methods return `MolyFuture` and `MolyStream` respectivly. These are basically boxed
dynamic futures/streams with some cross-platform stuff taken into consideration.

> **Note:** Boxed dynamic futures/streams are one of the things that can make async
> code in Rust difficult to write. This is because this kind of box erases important
> type information that Rust would normally use to make our lifes 10 times easier.
>
> However, since generics will not play well with Makepad widgets when they reach the
> UI, dynamic dispatching is a necesarly eveil.


The next thing to notice is that we can very easly create those kinds of futures/streams
from compile-time known futures by simply wrapping them with `moly_future(...)` and
`moly_stream(...)` functions.

We can also see these methods normally return a `ClientResult` which is slightly different
from Rust's standard `Result` as it can contain both, success recovered data and multiple
errors inside. However, the contructor methods of `ClientResult` enforces some semantics
similar to `Result`. If you use `new_ok(...)` it will contain just a sucess value. If you use
`new_err(...)` it will contain just an error. And if you use `new_ok_and_err` you are expected
to pass a non-empty list of errors along side your success data, otherwise a default error will
be inserted into the list and you probably don't want that.

Let's talk a little more about what the methods do. The `bots` methods simply returns
a list of available models/agents. It's pretty simple to implement, you could use
something like `reqwest` to fetch some JSON from your provider with the list of models,
parse than, and return it.

`send_stream` is the method that sends a message to a model/agent. It returns a
stream which allows you to just push chunks of content in real-time. Although, you
could also yield a single chunk from it if you don't support streaming.

Different than `Future`s in Rust, `Stream`s are a little less mature, so creating them
with the `async_stream` crate is advised for simplicity.

Note that the exact implementation of a client greatly depends on the provider you
are trying to support, so it's difficult to make a generic guide on how to build it
step by step. I recommend checking how `OpenAIClient` and `MultiClient` are implemented
as an example to do your own.

But to avoid leaving this section without a working client, let's finish the implementation
with some dummy methods for a client that simply repeats the last message.

```rust
struct EchoClient;

impl BotClient for EchoClient {
    fn bots(&self) -> MolyFuture<'static, ClientResult<Vec<Bot>>> {
        let future = futures::future::ready(ClientResult::new_ok(vec![Bot {
            id: BotId::new("echo-provider", "echo-echo"),
            name: "Echo Echo".to_string(),
            avatar: Picture::Grapheme("E".into()),
        }]));

        moly_future(future)
    }

    fn send_stream(
        &mut self,
        _bot: &Bot,
        messages: &[Message],
    ) -> MolyStream<'static, ClientResult<MessageContent>> {
        let last = messages.last().map(|m| m.content.text.clone()).unwrap_or_default();

        let stream = futures::stream::once(async move {
            ClientResult::new_ok(MessageContent {
                text: format!("Echo: {}", last),
                ..Default::default()
            })
        });

        moly_stream(stream)
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(EchoClient)
    }
}
```

## Warnings

### DO NOT `spawn` inside async methods

### Try not to use `tokio` specific code



## Reference

