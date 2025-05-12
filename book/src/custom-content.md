# Handle provider-specific content format
## Prerequisites
This guide assumes you have already read [Implement your own client](custom-client.md).

## Introduction

By default, Moly Kit displays `MessageContent` using the `StandardMessageContent`
widget.

This widget can render common content types returned by current LLMs,
such as simple markdown, thinking blocks, and citations from web searches.

However, a `BotClient` might interact with models or agents that return more complex
or unique content than what Moly Kit currently supports.

Therefore, `BotClient`s need a way to extend Moly Kit to render such unique
content.

This is quite straightforward. Clients can implement the `content_widget` method,
which allows them to return a custom UI widget to be rendered in place of the default
content widget, whenever the method deems it appropriate.

However, due to Makepad's architecture, users of your client must also perform
some "registration" steps for this to work.

In summary, the high-level steps are:
- Create a standard Makepad widget tailored to your content needs.
- Implement `content_widget` in your client. This method will create the widget
  using a template obtained by its ID.
- Instruct users of your client to register the widget manually, like any Makepad
  widget, using `live_design(cx)`.
- Instruct users of your client to create a template in Makepad's DSL and insert
  the `LivePtr` to that template under the expected ID.

## Detailed instructions with an example

Let's start by creating our custom content widget. This can be anything you need.
For this example, we'll implement one that simply displays text in a `Label`:

```rust
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub MyCustomContent = {{MyCustomContent}} {
        // It's important that height is set to Fit to avoid layout issues with Makepad.
        height: Fit,
        label = <Label> {}
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct MyCustomContent {
    #[deref]
    deref: View,
}

impl Widget for MyCustomContent {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope)
    }
}

impl MyCustomContent {
    pub fn set_content(&mut self, cx: &mut Cx, content: &MessageContent) {
        self.label(id!(label)).set_text(cx, &content.text);
    }
}
```

> **Note:** Making a `set_content` method that takes `MessageContent` is just a
> convention. It's not strictly necessary. You can design it however you want.

The next step is to implement the `content_widget` method in your `BotClient`:

```rust
impl BotClient for MyCustomClient {
    // ... other methods implemented ...

    fn content_widget(
        &mut self,
        cx: &mut Cx,
        previous_widget: WidgetRef,
        templates: &HashMap<LiveId, LivePtr>,
        content: &MessageContent,
    ) -> Option<WidgetRef> {
        // We expect the user of our client to register a template with the
        // id `MyCustomContent`.
        let Some(template) = templates.get(&live_id!(MyCustomContent)).copied() else {
            return None;
        };

        let Some(data) = content.data.as_deref() else {
            return None;
        };

        // Let's assume `MessageContent` yielded from our `send_stream` contains
        // this arbitrary data, explicitly stating it wants to be rendered with
        // `MyCustomContent`.
        if data != "I want to be displayed with MyCustomContent widget" {
          return None;
        }

        // If a widget already exists, let's try to reuse it to avoid losing
        // state.
        let widget = if previous_widget.as_my_custom_content().borrow().is_some() {
            previous_widget
        } else {
            // If the widget was not created yet, let's create it from the template
            // we obtained.
            WidgetRef::new_from_ptr(cx, Some(template))
        };

        // Let's call the `set_content` method we defined earlier to update the
        // content.
        widget
            .as_my_custom_content()
            .borrow_mut()
            .unwrap()
            .set_content(cx, content);

        Some(widget)
    }
}
```

Now, anyone who wants to use this client will need to register the widget like any
normal Makepad widget:

```rust
impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
        moly_kit::live_design(cx);

        // Add this line.
        my_custom_client::my_custom_content::live_design(cx);

        crate::widgets::live_design(cx);
    }
}
```

And finally, let's create a template for it and insert it into the `Chat` widget's
`Messages` component.

```rust
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    use moly_kit::widgets::chat::*;
    use my_custom_client::my_custom_content::*;

    pub MyCoolUi = {{MyCoolUi}} {
        // Notice the `:` here instead of `=`, to bind to the `#[live]` property
        // below.
        my_custom_content: <MyCustomContent> {}
        chat = <Chat> {}
    }
}

#[derive(Live, Widget)]
pub struct MyCoolUi {
    #[deref]
    deref: View,

    #[live]
    my_custom_content: LivePtr,
}

impl Widget for MyCoolUi {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope)
    }
}

impl LiveHook for MyCoolUi {
    // Let's insert the template as soon as the widget is created.
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        // ... other initialization code ...

        let chat = self.chat(id!(chat));
        let messages = chat.read().messages_ref();

        // We must use the ID that `content_widget` expects.
        messages.write().templates
            .insert(live_id!(MyCustomContent), self.my_custom_content);
    }
}
```

And that's it! All four pieces are in place. Now, whenever `content_widget`
returns `Some(a_custom_widget)`, `Messages` (the widget that renders the
list of messages inside `Chat`) will replace its default content widget with
the custom one.