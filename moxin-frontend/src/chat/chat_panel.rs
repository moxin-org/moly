use crate::chat::chat_line::*;
use crate::chat::model_selector::ModelSelectorAction;
use crate::data::chat::Chat;
use crate::data::store::Store;
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import makepad_draw::shader::std::*;

    import crate::chat::model_selector::ModelSelector;
    import crate::chat::chat_line::ChatLine;

    ICON_PROMPT = dep("crate://self/resources/icons/prompt.svg")

    ChatAgentAvatar = <RoundedView> {
        width: 20,
        height: 20,

        show_bg: true,
        draw_bg: {
            color: #444D9A
        }

        align: {x: 0.5, y: 0.5},

        avatar_label = <Label> {
            width: Fit,
            height: Fit,
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 10},
                color: #fff,
            }
            text: "P"
        }
    }

    UserChatLine = <ChatLine> {
        avatar_section = {
            <Image> {
                source: dep("crate://self/resources/images/chat_user_icon.png"),
                width: 20,
                height: 20,
            }
        }
    }

    ModelChatLine = <ChatLine> {
        avatar_section = {
            <ChatAgentAvatar> {}
        }
    }

    ChatPromptInput = <RoundedView> {
        width: Fill,
        height: 50,

        show_bg: true,
        draw_bg: {
            color: #fff
        }

        padding: {top: 3, bottom: 3, left: 4, right: 10}

        spacing: 4,
        align: {x: 0.0, y: 0.5},

        draw_bg: {
            radius: 2.0,
            border_color: #D0D5DD,
            border_width: 1.0,
        }

        prompt = <TextInput> {
            width: Fill,
            height: Fit,

            empty_message: "Search Model by Keyword"
            draw_bg: {
                color: #fff
            }
            draw_text: {
                text_style:<REGULAR_FONT>{font_size: 10},

                instance prompt_enabled: 0.0
                fn get_color(self) -> vec4 {
                    return mix(
                        #D0D5DD,
                        #000,
                        self.prompt_enabled
                    )
                }
            }

            // TODO find a way to override colors
            draw_cursor: {
                instance focus: 0.0
                uniform border_radius: 0.5
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(
                        0.,
                        0.,
                        self.rect_size.x,
                        self.rect_size.y,
                        self.border_radius
                    )
                    sdf.fill(mix(#fff, #bbb, self.focus));
                    return sdf.result
                }
            }

            // TODO find a way to override colors
            draw_select: {
                instance hover: 0.0
                instance focus: 0.0
                uniform border_radius: 2.0
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(
                        0.,
                        0.,
                        self.rect_size.x,
                        self.rect_size.y,
                        self.border_radius
                    )
                    sdf.fill(mix(#eee, #ddd, self.focus)); // Pad color
                    return sdf.result
                }
            }
        }

        prompt_icon = <RoundedView> {
            width: 30,
            height: 30,
            show_bg: true,
            draw_bg: {
                color: #D0D5DD
            }

            padding: {right: 4},
            align: {x: 0.5, y: 0.5},

            <Icon> {
                draw_icon: {
                    svg_file: (ICON_PROMPT),
                    fn get_color(self) -> vec4 {
                        return #fff;
                    }
                }
                icon_walk: {width: 12, height: 12}
            }
        }
    }

    ChatPanel = {{ChatPanel}} {
        width: Fill,
        height: Fill,
        margin: 20,
        spacing: 30,

        flow: Overlay,

        no_model = <View> {
            width: Fill,
            height: Fill,

            flow: Down,
            spacing: 30,
            align: {x: 0.5, y: 0.5},

            <Icon> {
                draw_icon: {
                    svg_file: dep("crate://self/resources/icons/chat.svg"),
                    color: #D0D5DD
                }
                icon_walk: {width: 128, height: 128}
            }
            <Label> {
                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 14},
                    color: #667085
                }
                text: "Start chatting by choosing a model from above"
            }
        }

        empty_conversation = <View> {
            visible: false,

            width: Fill,
            height: Fill,

            flow: Down,
            spacing: 30,
            align: {x: 0.5, y: 0.5},

            <ChatAgentAvatar> {}
            <Label> {
                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 14},
                    color: #101828
                }
                text: "How can I help you?"
            }
        }

        main = <View> {
            visible: false

            width: Fill,
            height: Fill,

            margin: { top: 60 }
            spacing: 30,
            flow: Down,

            chat = <PortalList> {
                width: Fill,
                height: Fill,

                auto_tail: true,

                UserChatLine = <UserChatLine> {}
                ModelChatLine = <ModelChatLine> {}
            }

            <ChatPromptInput> {}
        }

        <ModelSelector> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatPanel {
    #[deref]
    view: View,

    #[rust]
    loaded: bool,

    #[rust]
    auto_scroll_pending: bool,
    #[rust]
    auto_scroll_cancellable: bool,

    #[rust]
    is_chat_streaming: bool,
}

impl Widget for ChatPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        if let Event::Signal = event {
            let store = scope.data.get_mut::<Store>().unwrap();
            if let Some(chat) = &store.current_chat {
                self.auto_scroll_cancellable = true;
                let list = self.portal_list(id!(chat));

                self.is_chat_streaming = store.current_chat.as_ref().unwrap().is_streaming;
                if self.is_chat_streaming {
                    self.disable_prompt_input(cx);
                } else {
                    // Scroll to the bottom when streaming is done
                    self.scroll_messages_to_bottom(&list, chat);
                    self.auto_scroll_pending = false;
                    self.enable_prompt_input(cx);
                }

                if self.auto_scroll_pending {
                    // Scroll to the bottom
                    self.scroll_messages_to_bottom(&list, chat);
                }
            } else {
                //panic!("Unexpected error in the model chat session");
            }

            // Redraw because we expect to see new or updated chat entries
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>().unwrap();
        let chat_history;
        let model_filename;
        let initial_letter;

        if let Some(chat) = &store.current_chat {
            chat_history = chat.messages.clone();
            model_filename = chat.model_filename.clone();
            initial_letter = model_filename
                .chars()
                .next()
                .unwrap()
                .to_uppercase()
                .to_string();
        } else {
            chat_history = vec![];
            model_filename = "".to_string();
            initial_letter = "".to_string();
        };
        let chats_count = chat_history.len();

        if chats_count == 0 {
            self.view(id!(empty_conversation))
                .label(id!(avatar_label))
                .set_text(initial_letter.as_str());
        }

        while let Some(view_item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = view_item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, chats_count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < chats_count {
                        let chat_line_data = &chat_history[item_id];

                        let item;
                        let mut chat_line_item;
                        if chat_line_data.is_assistant() {
                            item = list.item(cx, item_id, live_id!(ModelChatLine)).unwrap();
                            chat_line_item = item.as_chat_line();
                            chat_line_item.set_role(&model_filename);
                            chat_line_item.set_avatar_text(&initial_letter);
                        } else {
                            item = list.item(cx, item_id, live_id!(UserChatLine)).unwrap();
                            chat_line_item = item.as_chat_line();
                            chat_line_item.set_role("You");
                        };

                        chat_line_item.set_message_text(&chat_line_data.content);
                        chat_line_item.set_message_id(chat_line_data.id);

                        // Disable actions for the last chat line when model is streaming
                        if self.is_chat_streaming && item_id == chats_count - 1 {
                            chat_line_item.set_actions_enabled(false);
                        } else {
                            chat_line_item.set_actions_enabled(true);
                        }

                        item.draw_all(cx, &mut Scope::empty());
                    }
                }
            }
        }

        DrawStep::done()
    }
}

impl WidgetMatchEvent for ChatPanel {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        for action in actions {
            match action.as_widget_action().cast() {
                ModelSelectorAction::Selected(downloaded_file) => {
                    self.loaded = true;
                    self.view(id!(main)).set_visible(true);
                    self.view(id!(empty_conversation)).set_visible(true);
                    self.view(id!(no_model)).set_visible(false);

                    let store = scope.data.get_mut::<Store>().unwrap();
                    store.load_model(&downloaded_file.file);
                    self.is_chat_streaming = false;
                }
                _ => {}
            }

            match action.as_widget_action().cast() {
                ChatLineAction::Delete(id) => {
                    let store = scope.data.get_mut::<Store>().unwrap();
                    store.delete_chat_message(id);
                    self.redraw(cx);
                }
                ChatLineAction::Edit(id, updated) => {
                    let store = scope.data.get_mut::<Store>().unwrap();
                    store.edit_chat_message(id, updated);
                    self.redraw(cx);
                }
                _ => {}
            }
        }

        if let Some(text) = self.text_input(id!(prompt)).changed(actions) {
            if !self.is_chat_streaming && text.len() > 0 {
                self.enable_prompt_input(cx);
            } else {
                self.disable_prompt_input(cx);
            }
        }

        let list = self.portal_list(id!(chat));
        if self.auto_scroll_cancellable && list.scrolled(actions) {
            // Cancel auto-scrolling if the user scrolls up
            self.auto_scroll_pending = false;
        }

        if !self.is_chat_streaming {
            if let Some(prompt) = self.text_input(id!(prompt)).returned(actions) {
                if prompt.trim().is_empty() {
                    return;
                }

                self.is_chat_streaming = true;
                self.disable_prompt_input(cx);
                let store = scope.data.get_mut::<Store>().unwrap();
                store.send_chat_message(prompt.clone());

                self.text_input(id!(prompt)).set_text_and_redraw(cx, "");
                self.view(id!(empty_conversation)).set_visible(false);

                // Scroll to the bottom when the message is sent
                if let Some(chat) = &store.current_chat {
                    self.scroll_messages_to_bottom(&list, chat);
                }
                self.auto_scroll_pending = true;
                self.auto_scroll_cancellable = false;
            }
        }
    }
}

impl ChatPanel {
    fn enable_prompt_input(&mut self, cx: &mut Cx) {
        let enabled_color = vec3(0.0, 0.0, 0.0);
        self.view(id!(prompt_icon)).apply_over(
            cx,
            live! {
                draw_bg: {
                    color: (enabled_color)
                }
            },
        );
        self.text_input(id!(prompt)).apply_over(
            cx,
            live! {
                draw_text: {
                    prompt_enabled: 1.0
                }
            },
        );
    }

    fn disable_prompt_input(&mut self, cx: &mut Cx) {
        let disabled_color = vec3(0.816, 0.835, 0.867); // #D0D5DD
        self.view(id!(prompt_icon)).apply_over(
            cx,
            live! {
                draw_bg: {
                    color: (disabled_color)
                }
            },
        );
        self.text_input(id!(prompt)).apply_over(
            cx,
            live! {
                draw_text: {
                    prompt_enabled: 0.0
                }
            },
        );
    }

    fn scroll_messages_to_bottom(&mut self, list: &PortalListRef, chat: &Chat) {
        if chat.messages.is_empty() {
            return;
        }
        list.set_first_id_and_scroll(chat.messages.len() - 1, 0.0);
    }
}
