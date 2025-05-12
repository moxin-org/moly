# Use multiple providers

## Prerequisites

This guide assumes you already read the [Quickstart](quickstart.md).

## Mixing clients

As seen before, a `BotRepo` is created from one (and only one) `BotClient`.

If we want out app to be able to use multiple clients configured in different ways
at the same time, we will need to compose them into one.

Fortunally, Moly Kit comes with a built-in client called `MultiClient`, which does
exactly that. `MultiClient` can take several "sub-clients" but acts as a single
one to `BotRepo`, routing requests to them accordengly.

Going back to our configuration from the [Quickstart](quickstart.md), we can
update it to work with several clients at the same time:

```rust
use moly_kit::*;

impl LiveHook for YourAmazingWidget {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        let client = {
          let mut client = MultiClient::new();

          let mut openai = OpenAIClient::new("https://api.openai.com/v1".into());
          openai.set_key("<YOUR_KEY>".into());
          client.add_client(openai);

          let mut openrouter = OpenAIClient::new("https://openrouter.ai/api/v1".into());
          openrouter.set_key("<YOUR_KEY>".into());
          client.add_client(openrouter);

          let ollama = OpenAIClient::new("http://localhost:11434/v1".into());
          client.add_client(ollama);

          client
        };

        let repo = BotRepo::from(client);

        let mut chat = self.chat(id!(chat));
        chat.write().bot_repo = repo;
    }
}
```