use makepad_widgets::*;
use moly_kit::*;
use moly_kit::utils::asynchronous::spawn;

use crate::data::capture::CaptureAction;
use crate::data::providers::ProviderType;
use crate::data::store::Store;
use crate::shared::actions::ChatAction;

use super::model_selector::ModelSelectorWidgetExt;
use super::model_selector_item::ModelSelectorAction;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::chat::chat_panel::ChatPanel;
    use crate::chat::chat_history::ChatHistory;
    use crate::chat::chat_params::ChatParams;
    use crate::chat::model_selector::ModelSelector;
    use moly_kit::widgets::chat::Chat;
    use moly_kit::widgets::prompt_input::PromptInput;

    PromptInputWithShadow = <PromptInput> {
        padding: {left: 10, right: 10, top: 8, bottom: 8}
        persistent = {
            // Shader to make the original RoundedView into a RoundedShadowView
            // (can't simply override the type of `persistent` because that removes the original children)
            clip_x:false, clip_y:false,
                                
            show_bg: true,
            draw_bg: {
                color: #f
                uniform border_radius: 5.0
                uniform border_size: 0.0
                uniform border_color: #0000
                uniform shadow_color: #0001
                uniform shadow_radius: 9.0,
                uniform shadow_offset: vec2(0.0,-2.5)
                                                
                varying rect_size2: vec2,
                varying rect_size3: vec2,
                varying rect_pos2: vec2,     
                varying rect_shift: vec2,    
                varying sdf_rect_pos: vec2,
                varying sdf_rect_size: vec2,
                                                
                fn get_color(self) -> vec4 {
                    return self.color
                }
                                                
                fn vertex(self) -> vec4 {
                    let min_offset = min(self.shadow_offset,vec2(0));
                    self.rect_size2 = self.rect_size + 2.0*vec2(self.shadow_radius);
                    self.rect_size3 = self.rect_size2 + abs(self.shadow_offset);
                    self.rect_pos2 = self.rect_pos - vec2(self.shadow_radius) + min_offset;
                    self.sdf_rect_size = self.rect_size2 - vec2(self.shadow_radius * 2.0 + self.border_size * 2.0)
                    self.sdf_rect_pos = -min_offset + vec2(self.border_size + self.shadow_radius);
                    self.rect_shift = -min_offset;
                                                                
                    return self.clip_and_transform_vertex(self.rect_pos2, self.rect_size3)
                }
                                                            
                fn get_border_color(self) -> vec4 {
                    return self.border_color
                }
                                                    
                fn pixel(self) -> vec4 {
                                                                    
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size3)
                    sdf.box(
                        self.sdf_rect_pos.x,
                        self.sdf_rect_pos.y,
                        self.sdf_rect_size.x,
                        self.sdf_rect_size.y, 
                        max(1.0, self.border_radius)
                    )
                    if sdf.shape > -1.0{
                        let m = self.shadow_radius;
                        let o = self.shadow_offset + self.rect_shift;
                        let v = GaussShadow::rounded_box_shadow(vec2(m) + o, self.rect_size2+o, self.pos * (self.rect_size3+vec2(m)), self.shadow_radius*0.5, self.border_radius*2.0);
                        sdf.clear(self.shadow_color*v)
                    }
                                                                        
                    sdf.fill_keep(self.get_color())
                    if self.border_size > 0.0 {
                        sdf.stroke(self.get_border_color(), self.border_size)
                    }
                    return sdf.result
                }
            }
        }
    }

    pub ChatScreen = {{ChatScreen}} {
        width: Fill,
        height: Fill,
        spacing: 10,

        <View> {
            width: Fit,
            height: Fill,

            chat_history = <ChatHistory> {}
        }

        <View> {
            width: Fill, height: Fill,
            align: {x: 0.5},
            padding: {top: 38, bottom: 10, right: 28, left: 28},
            flow: Down,
            spacing: 20

            model_selector = <ModelSelector> {}
            chat = <Chat> {
                prompt = <PromptInputWithShadow> {}
            }
        }

        // TODO: Add chat params back in, only when the model is a local model (MolyServer)
        // currenlty MolyKit does not support chat params
        // 
        // <View> {
        //     width: Fit,
        //     height: Fill,
        // 
        //     chat_params = <ChatParams> {}
        // }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatScreen {
    #[deref]
    view: View,

    #[rust(true)]
    first_render: bool,

    #[rust]
    should_load_repo_to_store: bool,

    #[rust]
    creating_bot_repo: bool,
}

impl Widget for ChatScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui_runner().handle(cx, event, scope, self);
        self.widget_match_event(cx, event, scope);
  
        // TODO This check is actually copied from Makepad view.rs file
        // It's not clear why it's needed here, but without this line
        // the "View all files" link in Discover section does not work after visiting the chat screen
        if self.visible || !event.requires_visibility() {
            self.view.handle_event(cx, event, scope);
        }

        let store = scope.data.get_mut::<Store>().unwrap();

        let should_recreate_bot_repo = store.bot_repo.is_none();

        if self.should_load_repo_to_store {
            store.bot_repo = self.chat(id!(chat)).read().bot_repo.clone();
            self.should_load_repo_to_store = false;
        } else if (self.first_render || should_recreate_bot_repo) && !self.creating_bot_repo {
            self.create_bot_repo(cx, scope);
            self.first_render = false;
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ChatScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();
        let mut chat_widget = self.chat(id!(chat));

        for action in actions {
            // Handle model selector actions
            match action.cast() {
                ModelSelectorAction::BotSelected(bot) => {
                    chat_widget.write().bot_id = Some(bot.id.clone());

                    if let Some(chat) = store.chats.get_current_chat() {
                        chat.borrow_mut().associated_bot = Some(bot.id.clone());
                        chat.borrow().save();
                    }
                    // self.focus_on_prompt_input_pending = true;
                }
                _ => {}
            }

            // Handle chat start
            match action.cast() {
                ChatAction::Start(bot_id) => {
                    store.chats.create_empty_chat(Some(bot_id.clone()));
                    self.messages(id!(chat.messages)).write().messages = vec![];
                    self.chat(id!(chat)).write().bot_id = Some(bot_id.clone());
                    self.model_selector(id!(model_selector)).set_currently_selected_model(Some(bot_id));
                    // self.focus_on_prompt_input_pending = true;
                }
                ChatAction::StartWithoutEntity => {
                    self.messages(id!(chat.messages)).write().messages = vec![];
                    // self.focus_on_prompt_input_pending = true;
                }
                _ => {}
            }

            // Hook into message updates to update the persisted chat history
            self.chat(id!(chat)).write_with(|chat| {
                let ui = self.ui_runner();
                chat.set_hook_after(move |group, _, _| {
                    for task in group.iter() {
                        // Handle new User messsages
                        if let ChatTask::InsertMessage(_index, message) = task {
                            let message = message.clone();
                            ui.defer_with_redraw(move |_me, _cx, scope| {
                                let current_chat = scope.data.get::<Store>().unwrap().chats.get_current_chat();
                                if let Some(store_chat) = current_chat {
                                    let mut store_chat = store_chat.borrow_mut();
                                    let mut new_message = message.clone();
                                    new_message.is_writing = false;
                                    store_chat.messages.push(new_message);
                                    store_chat.update_title_based_on_first_message();
                                    store_chat.save();
                                }
                            });
                        }

                        // Handle updated Bot messages
                        // UpdateMessage tasks mean that a bot message has been updated, either a User edit or a Bot message delta from the stream
                        // We fetch the current chat from the store and update the corresponding message, or insert it if it's not present 
                        // (if it's the first chunk from the bot message)
                        if let ChatTask::UpdateMessage(index, message) = task {
                            let message = message.clone();
                            let index = index.clone();
                            ui.defer_with_redraw(move |_me, _cx, scope| {
                                let current_chat = scope.data.get::<Store>().unwrap().chats.get_current_chat();
                                if let Some(store_chat) = current_chat {
                                    let mut store_chat = store_chat.borrow_mut();
                                    if let Some(message_to_update) = store_chat.messages.get_mut(index) {
                                        message_to_update.content = message.content.clone();
                                        message_to_update.is_writing = false;
                                    } else {
                                        let mut new_message = message.clone();
                                        new_message.is_writing = false;
                                        store_chat.messages.push(new_message);
                                    }
                                    store_chat.save();
                                }
                            });
                        }

                        if let ChatTask::DeleteMessage(index) = task {
                            let index = index.clone();
                            ui.defer_with_redraw(move |me, cx, scope| {
                                let store = scope.data.get_mut::<Store>().unwrap();
                                store.chats.delete_chat_message(index);
                                me.redraw(cx);
                            });
                        }

                        // TODO(MolyKit): Handle regenerate response?
                        //     ChatLineAction::Edit(id, updated, regenerate) => {
                        //         if regenerate {
                        //             self.send_message(cx, scope, updated, Some(id));
                        //             return;
                        //         } else {
                        //             store.edit_chat_message(id, updated);
                        //         }
                        //         self.redraw(cx);
                        //     }
                    }
                });
            });

            // Handle chat selection (from chat history)
            match action.cast() {
                ChatAction::ChatSelected(_chat_id) => {
                    let current_chat = store.chats.get_current_chat();

                    if let Some(chat) = current_chat {
                        store.preferences.set_current_chat_model(chat.borrow().associated_bot.clone());

                        // Load messages from history into the messages widget
                        self.messages(id!(chat.messages)).write().messages = chat.borrow().messages.clone();

                        // Set the chat's associated model in the model selector
                        if let Some(bot_id) = &chat.borrow().associated_bot {
                            self.model_selector(id!(model_selector)).set_currently_selected_model(Some(bot_id.clone()));
                            self.chat(id!(chat)).write().bot_id = Some(bot_id.clone());
                        }

                        self.redraw(cx);
                    }
                }
                _ => {}
            }

            // Handle Context Capture
            if let CaptureAction::Capture { event } = action.cast(){
                self.prompt_input(id!(prompt)).write().set_text(cx, event.contents());
            }
        }
    }
}


impl ChatScreen {
    fn create_bot_repo(&mut self, _cx: &mut Cx, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        let multi_client = {
            let mut multi_client = MultiClient::new();

            for provider in store.chats.providers.iter() {
                match provider.1.provider_type {
                    ProviderType::OpenAI | ProviderType::MolyServer => {
                        if provider.1.enabled && (provider.1.api_key.is_some() || provider.1.url.starts_with("http://localhost")) {
                            let mut new_client = OpenAIClient::new(provider.1.url.clone());
                            if let Some(key) = provider.1.api_key.as_ref() {
                                new_client.set_key(&key);
                            }
                            multi_client.add_client(Box::new(new_client));
                        }
                    },
                    ProviderType::MoFa => {
                        // For MoFa we don't require an API key
                        if provider.1.enabled {
                            let mut new_client = OpenAIClient::new(provider.1.url.clone());
                            if let Some(key) = provider.1.api_key.as_ref() {
                                new_client.set_key(&key);
                            }
                            multi_client.add_client(Box::new(new_client));
                        }
                    },
                    ProviderType::DeepInquire => {
                        let mut new_client = DeepInquireClient::new(provider.1.url.clone());
                        if let Some(key) = provider.1.api_key.as_ref() {
                            new_client.set_key(&key);
                        }
                        multi_client.add_client(Box::new(new_client));
                    }
                }
            }

            multi_client
        };
    
        let mut repo: BotRepo = multi_client.into();
        self.chat(id!(chat)).write().bot_repo = Some(repo.clone());

        self.creating_bot_repo = true;

        let ui = self.ui_runner();
            spawn(async move {
                repo.load().await;

                ui.defer_with_redraw(move |me, _cx, _scope| {
                me.should_load_repo_to_store = true;
                me.creating_bot_repo = false;
            });
        });
    }
}
