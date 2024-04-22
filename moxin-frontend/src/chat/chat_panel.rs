use crate::chat::chat_line::*;
use crate::chat::model_selector::ModelSelectorAction;
use crate::data::chat::Chat;
use crate::data::store::Store;
use crate::my_models::downloaded_files_table::DownloadedFileAction;
use makepad_widgets::*;
use moxin_protocol::data::DownloadedFile;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import makepad_draw::shader::std::*;

    import crate::chat::model_selector::ModelSelector;
    import crate::chat::chat_line::ChatLine;

    ICON_PROMPT = dep("crate://self/resources/icons/prompt.svg")
    ICON_STOP = dep("crate://self/resources/icons/stop.svg")
    ICON_JUMP_TO_BOTTOM = dep("crate://self/resources/icons/jump_to_bottom.svg")

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

    JumpToButtom = <View> {
        width: Fill,
        height: Fill,
        align: {x: 0.5, y: 1.0},

        jump_to_bottom = <CircleView> {
            width: 34,
            height: 34,
            align: {x: 0.5, y: 0.5},

            cursor: Hand,

            show_bg: true,

            draw_bg: {
                radius: 14.0,
                color: #fff,
                border_width: 2.0,
                border_color: #ccc,
            }

            <Icon> {
                padding: 0,
                // These margins are used to center the icon inside the circle
                // Not sure why the icon is not centered by default
                margin: { top: 6, right: 4 },
                draw_icon: {
                    svg_file: (ICON_JUMP_TO_BOTTOM),
                    fn get_color(self) -> vec4 {
                        return #1C1B1F;
                    }
                }
                icon_walk: {width: 12, height: 12}
            }
        }
    }

    ChatPromptInput = <RoundedView> {
        width: Fill,
        height: Fit,

        show_bg: true,
        draw_bg: {
            color: #fff
        }

        padding: {top: 6, bottom: 6, left: 4, right: 10}

        spacing: 4,
        align: {x: 0.0, y: 1.0},

        draw_bg: {
            radius: 2.0,
            border_color: #D0D5DD,
            border_width: 1.0,
        }

        prompt = <TextInput> {
            width: Fill,
            height: Fit,

            empty_message: "Enter a message"
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
            width: 28,
            height: 28,
            show_bg: true,
            draw_bg: {
                color: #D0D5DD
            }

            cursor: Hand,

            padding: {right: 4},
            margin: {bottom: 2},
            align: {x: 0.5, y: 0.5},

            icon_send = <View> {
                width: Fit,
                height: Fit,
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
            icon_stop = <View> {
                width: Fit,
                height: Fit,
                visible: false,

                <Icon> {
                    draw_icon: {
                        svg_file: (ICON_STOP),
                        fn get_color(self) -> vec4 {
                            return #fff;
                        }
                    }
                    icon_walk: {width: 12, height: 12}
                }
            }
        }
    }

    ChatPanel = {{ChatPanel}} {
        width: Fill,
        height: Fill,
        margin: {top: 0, left: 20, right: 20, bottom: 20},

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

            margin: { top: 86 }
            spacing: 4,
            flow: Down,

            <View> {
                width: Fill,
                height: Fill,

                flow: Overlay
                chat = <PortalList> {
                    width: Fill,
                    height: Fill,

                    drag_scrolling: false,
                    auto_tail: true,

                    UserChatLine = <UserChatLine> {}
                    ModelChatLine = <ModelChatLine> {}
                }

                <JumpToButtom> {}
            }

            <ChatPromptInput> {}
        }

        <ModelSelector> {}
    }
}

#[derive(Default, PartialEq)]
enum ChatPanelState {
    #[default]
    Unload,
    Idle,
    Streaming {
        auto_scroll_pending: bool,
        auto_scroll_cancellable: bool,
    },
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatPanel {
    #[deref]
    view: View,

    #[rust]
    state: ChatPanelState,
}

impl Widget for ChatPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        if let Event::Signal = event {
            let store = scope.data.get_mut::<Store>().unwrap();

            if let Some(chat) = &store.current_chat {
                match self.state {
                    ChatPanelState::Streaming {
                        auto_scroll_pending,
                        auto_scroll_cancellable: _,
                    } => {
                        self.state = ChatPanelState::Streaming {
                            auto_scroll_pending,
                            auto_scroll_cancellable: true,
                        };

                        let still_streaming = store.current_chat.as_ref().unwrap().is_streaming;
                        if still_streaming {
                            if auto_scroll_pending {
                                self.scroll_messages_to_bottom(chat);
                            }
                        } else {
                            // Scroll to the bottom when streaming is done
                            self.scroll_messages_to_bottom(chat);
                            self.state = ChatPanelState::Idle;
                        }

                        self.update_prompt_input(cx);

                        // Redraw because we expect to see new or updated chat entries
                        self.redraw(cx);
                    }
                    _ => {}
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>().unwrap();

        let (chat_history, model_filename, initial_letter) =
            store
                .current_chat
                .as_ref()
                .map_or((vec![], "".to_string(), "".to_string()), |chat| {
                    let model_filename = chat.model_filename.clone();
                    let initial_letter = model_filename
                        .chars()
                        .next()
                        .unwrap_or_default()
                        .to_uppercase()
                        .to_string();
                    (chat.messages.clone(), model_filename, initial_letter)
                });

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
                        if matches!(self.state, ChatPanelState::Streaming { .. })
                            && item_id == chats_count - 1
                        {
                            chat_line_item.set_actions_enabled(cx, false);
                        } else {
                            chat_line_item.set_actions_enabled(cx, true);
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
                    let store = scope.data.get_mut::<Store>().unwrap();
                    self.load_model(store, downloaded_file);
                }
                _ => {}
            }

            match action.as_widget_action().cast() {
                DownloadedFileAction::StartChat(downloaded_file) => {
                    let store = scope.data.get_mut::<Store>().unwrap();
                    self.load_model(store, downloaded_file);
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

        self.jump_to_bottom_actions(cx, actions, scope);

        match self.state {
            ChatPanelState::Idle => {
                self.handle_prompt_input_actions(cx, actions, scope);
            }
            ChatPanelState::Streaming {
                auto_scroll_pending: _,
                auto_scroll_cancellable,
            } => {
                let list = self.portal_list(id!(chat));
                if auto_scroll_cancellable && list.scrolled(actions) {
                    // Cancel auto-scrolling if the user scrolls up
                    self.state = ChatPanelState::Streaming {
                        auto_scroll_pending: false,
                        auto_scroll_cancellable,
                    };
                }

                if let Some(fe) = self.view(id!(prompt_icon)).finger_up(&actions) {
                    if fe.was_tap() {
                        let store = scope.data.get_mut::<Store>().unwrap();
                        store.cancel_chat_streaming();
                    }
                }
            }
            ChatPanelState::Unload => {}
        }
    }
}

impl ChatPanel {
    fn jump_to_bottom_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if let Some(fe) = self.view(id!(jump_to_bottom)).finger_up(actions) {
            if fe.was_tap() {
                let store = scope.data.get_mut::<Store>().unwrap();
                if let Some(chat) = &store.current_chat {
                    self.scroll_messages_to_bottom(chat);
                    self.redraw(cx);
                }
            }
        }

        let jump_to_bottom = self.view(id!(jump_to_bottom));
        match self.state {
            ChatPanelState::Idle | ChatPanelState::Streaming { .. } => {
                let store = scope.data.get_mut::<Store>().unwrap();
                let has_messages = store
                    .current_chat
                    .as_ref()
                    .map_or(false, |chat| chat.messages.len() > 0);

                // TODO make it visible only when scrolling up
                // (we need to improve PortalList API for this)
                jump_to_bottom.set_visible(has_messages);
            }
            ChatPanelState::Unload => {
                jump_to_bottom.set_visible(false);
            }
        }
    }

    fn update_prompt_input(&mut self, cx: &mut Cx) {
        match self.state {
            ChatPanelState::Idle => {
                self.enable_or_disable_prompt_input(cx);
                self.show_prompt_input_send_icon(cx);
            }
            ChatPanelState::Streaming {
                auto_scroll_pending: _,
                auto_scroll_cancellable: _,
            } => {
                let prompt_input = self.text_input(id!(prompt));
                prompt_input.apply_over(
                    cx,
                    live! {
                        draw_text: { prompt_enabled: 0.0 }
                    },
                );
                self.show_prompt_input_stop_icon(cx);
            }
            ChatPanelState::Unload => {}
        }
    }

    fn enable_or_disable_prompt_input(&mut self, cx: &mut Cx) {
        let prompt_input = self.text_input(id!(prompt));
        let enable = if prompt_input.text().len() > 0 {
            1.0
        } else {
            0.0
        };

        prompt_input.apply_over(
            cx,
            live! {
                draw_text: { prompt_enabled: (enable) }
            },
        );
    }

    fn show_prompt_input_send_icon(&mut self, cx: &mut Cx) {
        self.view(id!(prompt_icon)).apply_over(
            cx,
            live! {
                icon_send = { visible: true }
                icon_stop = { visible: false }
            },
        );
        let prompt_input = self.text_input(id!(prompt));
        if prompt_input.text().len() > 0 {
            self.enable_prompt_input_icon(cx);
        } else {
            self.disable_prompt_input_icon(cx);
        }
    }

    fn show_prompt_input_stop_icon(&mut self, cx: &mut Cx) {
        self.view(id!(prompt_icon)).apply_over(
            cx,
            live! {
                icon_send = { visible: false }
                icon_stop = { visible: true }
            },
        );
        self.enable_prompt_input_icon(cx);
    }

    fn enable_prompt_input_icon(&mut self, cx: &mut Cx) {
        let enabled_color = vec3(0.0, 0.0, 0.0);
        self.view(id!(prompt_icon)).apply_over(
            cx,
            live! {
                draw_bg: {
                    color: (enabled_color)
                }
            },
        );
    }

    fn disable_prompt_input_icon(&mut self, cx: &mut Cx) {
        let disabled_color = vec3(0.816, 0.835, 0.867); // #D0D5DD
        self.view(id!(prompt_icon)).apply_over(
            cx,
            live! {
                draw_bg: {
                    color: (disabled_color)
                }
            },
        );
    }

    fn scroll_messages_to_bottom(&mut self, chat: &Chat) {
        if chat.messages.is_empty() {
            return;
        }
        let list = self.portal_list(id!(chat));
        list.set_first_id_and_scroll(chat.messages.len() - 1, 0.0);
    }

    fn load_model(&mut self, store: &mut Store, downloaded_file: DownloadedFile) {
        self.state = ChatPanelState::Idle;
        self.view(id!(main)).set_visible(true);
        self.view(id!(empty_conversation)).set_visible(true);
        self.view(id!(no_model)).set_visible(false);

        store.load_model(&downloaded_file.file);
    }

    fn handle_prompt_input_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let prompt_input = self.text_input(id!(prompt));

        if let Some(_text) = prompt_input.changed(actions) {
            self.update_prompt_input(cx);
        }

        if let Some(fe) = self.view(id!(prompt_icon)).finger_up(&actions) {
            if fe.was_tap() {
                self.send_message(cx, scope, prompt_input.text());
            }
        }

        if let Some(prompt) = prompt_input.returned(actions) {
            self.send_message(cx, scope, prompt);
        }
    }

    fn send_message(&mut self, cx: &mut Cx, scope: &mut Scope, prompt: String) {
        if prompt.trim().is_empty() {
            return;
        }

        self.show_prompt_input_stop_icon(cx);
        let store = scope.data.get_mut::<Store>().unwrap();
        store.send_chat_message(prompt.clone());

        let prompt_input = self.text_input(id!(prompt));
        prompt_input.set_text_and_redraw(cx, "");
        prompt_input.set_cursor(0, 0);
        self.update_prompt_input(cx);

        self.view(id!(empty_conversation)).set_visible(false);

        // Scroll to the bottom when the message is sent
        if let Some(chat) = &store.current_chat {
            self.scroll_messages_to_bottom(chat);
        }

        self.state = ChatPanelState::Streaming {
            auto_scroll_pending: true,
            auto_scroll_cancellable: false,
        };
    }
}
