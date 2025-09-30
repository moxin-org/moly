use crate::controllers::chat::{
    ChatController, ChatControllerPlugin, ChatControllerPluginRegistrationId, ChatTask,
};
use crate::utils::makepad::EventExt;
use crate::*;
use makepad_widgets::*;
use std::cell::{Ref, RefMut};
use std::sync::{Arc, Mutex};

live_design!(
    use link::theme::*;
    use link::widgets::*;
    use link::moly_kit_theme::*;
    use link::shaders::*;

    use crate::widgets::messages::*;
    use crate::widgets::prompt_input::*;
    use crate::widgets::moly_modal::*;
    use crate::widgets::realtime::*;

    pub Chat = {{Chat}} <RoundedView> {
        flow: Down,
        messages = <Messages> {}
        prompt = <PromptInput> {}

        <View> {
            width: Fill, height: Fit
            flow: Overlay

            audio_modal = <MolyModal> {
                dismiss_on_focus_lost: false
                content: <RoundedView> {
                    draw_bg: {border_radius: 10}
                    width: 450, height: Fit
                    align: {x: 0.5, y: 0.5}
                    realtime = <Realtime>{}
                }
            }
        }
    }
);
/// A batteries-included chat to to implement chatbots.
#[derive(Live, LiveHook, Widget)]
pub struct Chat {
    #[deref]
    deref: View,

    #[rust]
    pub controller: Option<Arc<Mutex<ChatController>>>,

    #[rust]
    plugin_id: Option<ChatControllerPluginRegistrationId>,
}

impl Widget for Chat {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Pass down the controller if not the same.
        self.messages_ref().write_with(|m| {
            let a = self.controller.as_ref().map(|c| Arc::as_ptr(c) as usize);
            let b = m.controller.as_ref().map(|c| Arc::as_ptr(c) as usize);
            if a != b {
                m.controller = self.controller.clone();
            }
        });

        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope);
        self.handle_messages(cx, event);
        self.handle_prompt_input(cx, event);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.plugin_id.is_none() {
            let ui_runner = self.ui_runner();
            let plugin = ChatPlugin(ui_runner);
            let id = self.controller_lock().register_plugin(plugin);
            self.plugin_id = Some(id);
        }

        if self.controller_lock().state().load_status.is_idle() {
            self.controller_lock().dispatch_task(ChatTask::Load);
        }

        self.deref.draw_walk(cx, scope, walk)
    }
}

impl Chat {
    /// Getter to the underlying [PromptInputRef] independent of its id.
    pub fn prompt_input_ref(&self) -> PromptInputRef {
        self.prompt_input(id!(prompt))
    }

    /// Getter to the underlying [MessagesRef] independent of its id.
    pub fn messages_ref(&self) -> MessagesRef {
        self.messages(id!(messages))
    }

    fn handle_prompt_input(&mut self, cx: &mut Cx, event: &Event) {
        if self.prompt_input_ref().read().submitted(event.actions()) {
            self.handle_submit(cx);
        }
    }

    fn handle_messages(&mut self, cx: &mut Cx, event: &Event) {
        for action in event.actions() {
            let Some(action) = action.as_widget_action() else {
                continue;
            };

            if action.widget_uid != self.messages_ref().widget_uid() {
                continue;
            }

            match action.cast::<MessagesAction>() {
                MessagesAction::Delete(index) => {}
                MessagesAction::Copy(index) => {}
                MessagesAction::EditSave(index) => {}
                MessagesAction::EditRegenerate(index) => {}
                MessagesAction::ToolApprove(index) => {}
                MessagesAction::ToolDeny(index) => {}
                MessagesAction::None => {}
            }
        }
    }

    fn handle_submit(&mut self, cx: &mut Cx) {
        let prompt = self.prompt_input_ref();

        if prompt.read().has_send_task() {
            // TODO: Decide if prompt input should be binded before.
            let text = prompt.text();
            let attachments = prompt
                .read()
                .attachment_list_ref()
                .read()
                .attachments
                .clone();

            if !text.is_empty() || !attachments.is_empty() {
                // TODO: Clearing the text was hookable before.
                prompt.set_text(cx, "");
                self.controller_lock()
                    .dispatch_state_mutation(move |state| {
                        let content = MessageContent {
                            text: text.clone(),
                            attachments: attachments.clone(),
                            ..Default::default()
                        };
                        state.prompt_input_content = content;
                    });
                self.controller_lock().dispatch_task(ChatTask::Send);
            }
        } else if prompt.read().has_stop_task() {
            self.controller_lock().dispatch_task(ChatTask::Stop);
        }
    }

    fn controller_lock(&self) -> std::sync::MutexGuard<'_, ChatController> {
        self.controller
            .as_ref()
            .expect("Chat controller not set")
            .lock()
            .unwrap()
    }
}

// TODO: Since `ChatRef` is generated by a macro, I can't document this to give
// these functions better visibility from the module view.
impl ChatRef {
    /// Immutable access to the underlying [Chat].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read(&self) -> Ref<Chat> {
        self.borrow().unwrap()
    }

    /// Mutable access to the underlying [Chat].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write(&mut self) -> RefMut<Chat> {
        self.borrow_mut().unwrap()
    }

    /// Immutable reader to the underlying [Chat].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read_with<R>(&self, f: impl FnOnce(&Chat) -> R) -> R {
        f(&*self.read())
    }

    /// Mutable writer to the underlying [Chat].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write_with<R>(&mut self, f: impl FnOnce(&mut Chat) -> R) -> R {
        f(&mut *self.write())
    }
}

struct ChatPlugin(UiRunner<Chat>);

impl ChatControllerPlugin for ChatPlugin {
    fn on_state_change(&mut self, state: &controllers::chat::ChatState) {
        self.0.defer_with_redraw(|_, _, _| {});
    }
}
