# Integrate and customize behavior
## Prerequisites

This guide assumes you have already read the [Quickstart](quickstart.md).

## Introduction

As we saw before, we only need to give a `BotContext` to `Chat`, and then it just works.
The `Chat` widget is designed to work on its own after initial configuration, without
you needing to do anything else.

However, when integrating it into your own complex apps, you will eventually need
to take control of specific things. `Chat` allows you to do just that in a way
tailored to your needs through a "hooking" system.

Since this is an important concept to understand, let's start by explaining the
theory.

## Hooks and tasks

When `Chat` wants to perform a relevant interaction, like sending a message, updating
texts, copying to the clipboard, etc., it will NOT do that without giving you the chance
to take control.

To do so, those relevant interactions are defined as "tasks", and they are grouped
and emitted to a callback that runs just before `Chat` executes the action.

That callback is what we define as a "hook". A hook not only gets notified
of what is about to happen, but also gets mutable access to the group of tasks,
meaning it can modify them or abort them as needed.

> So, in other words, "tasks" are "units of work" that are about to be performed,
> but we can "tamper" with them.

## The `set_hook_before` method

The `set_hook_before` method can be used during the configuration of the `Chat` widget
to set a closure that will run just before `Chat` performs any relevant action.

It will receive a `&mut Vec<ChatTask>` as its first parameter, which is the representation
of the actions that will be performed as part of a group.

`Chat` uses the information from inside a `ChatTask` to perform the real action,
so modifying their data will impact how the action is executed.

Additionally, as this is exposed as a simple mutable vector, you can `clear()` it
to essentially prevent `Chat` from doing anything with them.

> This is basically an "abort" mechanism, similar to a web browser's `preventDefault()`
> method in `Event`.

Of course, you can do anything you want with this vector, like injecting more
tasks, swapping them, etc.

Enough theory. Let's see a practical example. We will modify the setup code from
the [Quickstart](quickstart.md) to configure a hook:

```rust
use moly_kit::*;

impl LiveHook for YourAmazingWidget {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        // ... the previous setup code from quickstart ...

        chat.write().set_hook_before(|group, chat, cx| {
          // If we set this to true, the group of tasks will be cleared and
          // the default behavior will not happen.
          let mut abort = false;

          // We don't care about grouping right now so let's just deal with them
          // individually.
          for task in group.iter_mut() {
            // Let's log to the console when sending a message.
            if let ChatTask::Send = task {
              println!("A message is being sent!");
            }

            // Let's ensure a message always gets updated with uppercase text.
            if let ChatTask::UpdateMessage(_index, message) = task {
              message.content.text = message.content.text.to_uppercase();
            }

            // Let's prevent the default behavior of copying a message to the clipboard
            // and handle it ourselves to add a watermark.
            if let ChatTask::CopyMessage(index) = task {
              abort = true;

              let messages = chat.messages_ref();
              let text = messages.read().messages[*index].content.text.clone();
              let text = format!("You copied the following text: {}", text);

              cx.copy_to_clipboard(&text);
            }
          }

          if abort {
            group.clear();
          }
        });
    }
}
```

Okay, that was a very comprehensive example that is worth a hundred words.

## Why are tasks grouped?

Tasks are grouped because not all UI interactions map to a single unit
of work.

For example, the "Save and regenerate" button will trigger two grouped tasks:
`SetMessages` to override the message history, and then `Send` to send the history
as it is.

> Notice how `Send` doesn't take parameters, as it just sends the whole message history.
> We try to keep task responsibilities decoupled so you don't need to handle many
> similar tasks to handle some common/intercepted behavior.

If these tasks were emitted individually, then it would be difficult to inspect this
kind of detail, and you might abort a single task that a future task was expecting
to exist.

## How do I leak data out of the hook closure?

Due to a hook being a closure executed by `Chat` at an unknown time for your parent
widget, and because of Rust's lifetimes, we can't just directly access data from
the outer scope of the closure.

So we need to communicate back with our parent widget somehow. We will not cover
this with detailed examples as you are probably familiar with how to communicate
Makepad widgets, but here are some ideas of what you can do.

Since you have access to `cx` inside the closure, one way would be to emit an action
with `cx.widget_action(...)` and receive it in your parent widget or use `cx.action(...)`
and handle it globally.

If you don't want to define messages and instead you want to directly execute something
in the context of your widget, you can use Makepad's `UiRunner` to send instructions
packed in a closure back to your parent widget.

## What interactions can I intercept with this?

Any interaction listed and documented in the `ChatTask` enum can be worked with from
inside the hook.

You may want to read that enum's specific documentation in the crate documentation.
