use makepad_widgets::*;
use std::cell::{Ref, RefCell, RefMut};

use crate::{
    chat::{
        chat_line::{ChatLineAction, ChatLineWidgetRefExt},
        model_selector_item::ModelSelectorAction,
    },
    data::{
        chats::{
            chat::{Chat, ChatEntityAction, ChatID, ChatMessage},
            chat_entity::ChatEntityId,
        },
        store::Store,
    },
    shared::actions::ChatAction,
};

use super::{
    model_selector_list::ModelSelectorListAction, prompt_input::PromptInputWidgetExt,
    shared::ChatAgentAvatarWidgetRefExt,
};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::chat::model_selector::ModelSelector;
    use crate::chat::chat_line::ChatLine;
    use crate::chat::shared::ChatAgentAvatar;
    use crate::chat::shared::ChatModelAvatar;
    use crate::chat::prompt_input::PromptInput;

    ICON_JUMP_TO_BOTTOM = dep("crate://self/resources/icons/jump_to_bottom.svg")

    CircleButton = <MolyButton> {
        padding: {right: 2},
        margin: {bottom: 2},

        draw_icon: {
            color: #fff
        }
        icon_walk: {width: 12, height: 12}
    }

    UserChatLine = <ChatLine> {
        margin: {left: 100}
        avatar_section = {
            visible: false,
        }
        main_section = {
            body_section = {
                align: {x: 1.0, y: 0.5},
                sender_name_layout = {
                    visible: false,
                }
                bubble = {
                    draw_bg: {
                        color: #15859A
                    }
                    markdown_message_container = {
                        markdown_message = {
                            font_color: #fff,
                            draw_normal: {
                                color: #fff,
                            }
                            draw_italic: {
                                color: #fff,
                            }
                            draw_bold: {
                                color: #fff,
                            }
                            draw_bold_italic: {
                                color: #fff,
                            }
                            draw_fixed: {
                                color: #fff,
                            }
                            draw_block: {
                                line_color: #fff
                                sep_color: #12778a
                                quote_bg_color: #12778a
                                quote_fg_color: #106a7b
                                code_color: #12778a
                            }
                        }
                    }
                    plain_text_message_container = {
                        plain_text_message = {
                            draw_text: {
                                color: #fff
                            }
                        }
                    }
                    edit_buttons = {
                        save = {
                            draw_bg: { border_color: #D0D5DD, border_width: 1.0, color: #fff }
                            draw_text: {
                                fn get_color(self) -> vec4 {
                                    return #099250;
                                }
                            }
                        }
                        save_and_regenerate = {
                            draw_bg: { border_color: #D0D5DD, border_width: 1.0, color: #fff }
                            draw_text: {
                                fn get_color(self) -> vec4 {
                                    return #099250;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    ModelChatLine = <ChatLine> {
        main_section = {
            body_section = {
                bubble = {
                    padding: {left: 0, bottom: 0, top: 0}
                    markdown_message_container = {
                        markdown_message = {
                            draw_normal: {
                                color: (#000),
                            }
                            draw_italic: {
                                color: (#000),
                            }
                            draw_bold: {
                                color: (#000),
                            }
                            draw_bold_italic: {
                                color: (#000),
                            }
                            draw_fixed: {
                                color: (#000),
                            }
                            draw_block: {
                                line_color: (#000)
                                sep_color: (#EDEDED)
                                quote_bg_color: (#EDEDED)
                                quote_fg_color: (#969696)
                                code_color: (#EDEDED)
                            }
                        }
                    }
                }
            }
        }
    }

    JumpToBottom = <View> {
        width: Fill,
        height: Fill,
        align: {x: 1.0, y: 1.0},
        padding: {bottom: 60},

        jump_to_bottom = <CircleButton> {
            width: 34,
            height: 34,
            margin: {bottom: 10},

            draw_bg: {
                radius: 8.0,
                color: #fff,
                border_width: 1.0,
                border_color: #EAECF0,
            }
            draw_icon: {
                svg_file: (ICON_JUMP_TO_BOTTOM),
                fn get_color(self) -> vec4 {
                    return #1C1B1F;
                }
            }
            icon_walk: {
                margin: {top: 6, left: -4},
            }
        }
    }

    pub ChatPanel = {{ChatPanel}} {
        flow: Overlay
        width: Fill
        height: Fill

        <View> {
            flow: Overlay
            width: Fill
            height: Fill
            padding: {left: 25, right: 25, bottom: 20},

            no_downloaded_model = <View> {
                visible: false,
                width: Fill,
                height: Fill,

                flow: Down,
                align: {x: 0.5, y: 0.5},

                <View> {
                    width: Fill,
                    height: Fill,
                    flow: Down,
                    spacing: 30,
                    align: {x: 0.5, y: 0.5},

                    <Label> {
                        draw_text: {
                            text_style: <REGULAR_FONT>{font_size: 12},
                            color: #667085
                        }
                        text: "You haven’t downloaded any models yet."
                    }
                    go_to_discover_button = <MolyButton> {
                        width: Fit,
                        height: Fit,

                        draw_bg: {
                            border_color: #D0D5DD,
                            border_width: 1.0,
                            color: #fff,
                            color_hover: #E2F1F1,
                            radius: 2.0,
                        }

                        padding: {top: 14, right: 12, bottom: 14, left: 12}
                        text: "Go To Discover"
                        draw_text: {
                            text_style: <BOLD_FONT>{font_size: 12},
                            fn get_color(self) -> vec4 {
                                return #087443;
                            }
                        }
                    }
                }

                <View> {
                    width: Fill, height: Fit
                    flow: Down,
                    align: {x: 0.5, y: 0.5},
                    no_downloaded_model_prompt_input = <PromptInput> {}
                }

            }

            no_model = <View> {
                visible: false,
                width: Fill,
                height: Fill,

                flow: Down,
                align: {x: 0.5, y: 0.5},

                <View> {
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

                <View> {
                    width: Fill, height: Fit
                    flow: Down,
                    align: {x: 0.5, y: 0.5},
                    no_model_prompt_input = <PromptInput> {}
                }

            }

            empty_conversation = <View> {
                visible: false,

                width: Fill,
                height: Fill,

                flow: Down,
                spacing: 30,
                align: {x: 0.5, y: 0.5},

                avatar_section = <View> {
                    width: Fit, height: Fit,
                    model = <ChatModelAvatar> {}
                    agent = <ChatAgentAvatar> { visible: false }
                }

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

                chat = <PortalList> {
                    margin: { bottom: 15 }
                    scroll_bar: {
                        bar_size: 0.0,
                    }
                    width: Fill,
                    height: Fill,

                    drag_scrolling: false,

                    UserChatLine = <UserChatLine> {}
                    ModelChatLine = <ModelChatLine> {}
                    EndOfChat = <View> {height: 0.1}
                }


                main_prompt_input = <PromptInput> {}
            }

            model_selector = <ModelSelector> {}
        }

        <JumpToBottom> {}
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default)]
enum State {
    /// `Unknown` is simply the default state, meaning the state has not been loaded yet,
    /// and therefore indicates a development error if it is encountered.
    #[default]
    Unknown,
    NoModelsAvailable,
    NoModelSelected,
    ModelSelectedWithEmptyChat {
        is_loading: bool,
    },
    ModelSelectedWithChat {
        is_loading: bool,
        sticked_to_bottom: bool,
        receiving_response: bool,
        was_cancelled: bool,
    },
}

enum PromptInputMode {
    Enabled,
    Disabled,
}
#[derive(Debug)]
enum PromptInputButton {
    Send,
    EnabledStop,
    DisabledStop,
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatPanel {
    #[deref]
    view: View,

    #[rust]
    state: State,

    #[rust]
    portal_list_end_reached: bool,

    #[rust(false)]
    focus_on_prompt_input_pending: bool,

    #[rust]
    current_chat_id: Option<ChatID>,
}

impl Widget for ChatPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        if store.chats.get_current_chat_id() != self.current_chat_id {
            self.current_chat_id = store.chats.get_current_chat_id();
            self.reset_scroll_messages(store);
            self.redraw(cx);
        }

        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        self.update_state(scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.update_view(cx, scope);

        // We need to make sure we're drawing this widget in order to focus on the prompt input
        // Otherwise, when navigating from another section this command would happen before the widget is drawn
        // (not having any effect).
        if self.focus_on_prompt_input_pending {
            self.focus_on_prompt_input_pending = false;

            self.prompt_input(id!(main_prompt_input)).reset_text(true);
        }

        let message_list_uid = self.portal_list(id!(chat)).widget_uid();
        while let Some(view_item) = self.view.draw_walk(cx, scope, walk).step() {
            if view_item.widget_uid() == message_list_uid {
                self.draw_messages(
                    cx,
                    scope,
                    &mut view_item.as_portal_list().borrow_mut().unwrap(),
                );
            }
        }

        DrawStep::done()
    }
}

impl WidgetMatchEvent for ChatPanel {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        for action in actions {
            if let Some(action) = action.downcast_ref::<ChatEntityAction>() {
                if get_chat_id(store) == Some(action.chat_id) {
                    match self.state {
                        State::ModelSelectedWithChat {
                            receiving_response: true,
                            sticked_to_bottom,
                            ..
                        } => {
                            if sticked_to_bottom {
                                self.scroll_messages_to_bottom(cx);
                            }

                            // Redraw because we expect to see new or updated chat entries
                            self.redraw(cx);
                        }
                        _ => {}
                    }
                }
            }

            match action.cast() {
                ModelSelectorAction::ModelSelected(downloaded_file) => {
                    store.load_model(&downloaded_file.file);

                    if let Some(chat) = store.chats.get_current_chat() {
                        chat.borrow_mut().associated_entity =
                            Some(ChatEntityId::ModelFile(downloaded_file.file.id.clone()));
                        chat.borrow().save();
                    }

                    self.focus_on_prompt_input_pending = true;
                    self.redraw(cx)
                }
                ModelSelectorAction::AgentSelected(agent) => {
                    if let Some(chat) = store.chats.get_current_chat() {
                        chat.borrow_mut().associated_entity = Some(ChatEntityId::Agent(agent.id));
                        chat.borrow().save();
                    }

                    self.focus_on_prompt_input_pending = true;
                    self.redraw(cx);
                }
                _ => {}
            }
        }

        for action in actions.iter() {
            if let ModelSelectorListAction::AddedOrDeletedModel = action.cast() {
                self.redraw(cx);
            }

            match action.cast() {
                ChatAction::Start(handler) => match handler {
                    ChatEntityId::ModelFile(file_id) => {
                        if let Some(file) = store.downloads.get_file(&file_id) {
                            store.chats.create_empty_chat_and_load_file(file);
                            self.focus_on_prompt_input_pending = true;
                        }
                    }
                    ChatEntityId::Agent(agent_id) => {
                        store.chats.create_empty_chat_with_agent(&agent_id);
                        self.focus_on_prompt_input_pending = true;
                    }
                },
                _ => {}
            }

            match action.cast() {
                ChatLineAction::Delete(id) => {
                    store.chats.delete_chat_message(id);
                    self.redraw(cx);
                }
                ChatLineAction::Edit(id, updated, regenerate) => {
                    if regenerate {
                        self.send_message(cx, scope, updated, Some(id));
                        return;
                    } else {
                        store.edit_chat_message(id, updated);
                    }
                    self.redraw(cx);
                }
                _ => {}
            }
        }

        if self.button(id!(jump_to_bottom)).clicked(actions) {
            self.scroll_messages_to_bottom(cx);
            self.redraw(cx);
        }

        match self.state {
            State::ModelSelectedWithChat {
                receiving_response: false,
                ..
            }
            | State::ModelSelectedWithEmptyChat { .. } => {
                self.handle_prompt_input_actions(cx, actions, scope);
            }
            State::ModelSelectedWithChat {
                receiving_response: true,
                ..
            } => {
                if self
                    .button(id!(main_prompt_input.prompt_stop_button))
                    .clicked(actions)
                {
                    store.chats.cancel_chat_streaming();
                }
            }
            _ => {}
        }

        if self
            .button(id!(no_downloaded_model.go_to_discover_button))
            .clicked(actions)
        {
            cx.action(ChatPanelAction::NavigateToDiscover);
        }
    }
}

impl ChatPanel {
    fn update_state(&mut self, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        let chat_entity = store
            .chats
            .get_current_chat()
            .and_then(|c| c.borrow().associated_entity.clone());

        self.state = if chat_entity.is_none() && store.chats.loaded_model.is_none() {
            State::NoModelSelected
        } else {
            // Model or Agent is selected
            let is_loading = store.chats.model_loader.is_loading();

            store.chats.get_current_chat().map_or(
                State::ModelSelectedWithEmptyChat { is_loading },
                |chat| {
                    if chat.borrow().messages.is_empty() {
                        State::ModelSelectedWithEmptyChat { is_loading }
                    } else {
                        State::ModelSelectedWithChat {
                            is_loading,
                            sticked_to_bottom: self.portal_list_end_reached
                                || !matches!(self.state, State::ModelSelectedWithChat { .. }),
                            receiving_response: chat.borrow().is_receiving(),
                            was_cancelled: chat.borrow().was_cancelled(),
                        }
                    }
                },
            )
        }
    }

    fn update_prompt_input(&mut self, cx: &mut Cx) {
        match self.state {
            State::ModelSelectedWithEmptyChat { is_loading: true }
            | State::ModelSelectedWithChat {
                is_loading: true, ..
            } => {
                self.activate_prompt_input(cx, PromptInputMode::Disabled, PromptInputButton::Send);
            }
            State::ModelSelectedWithEmptyChat { is_loading: false }
            | State::ModelSelectedWithChat {
                is_loading: false,
                receiving_response: false,
                was_cancelled: false,
                ..
            } => {
                self.activate_prompt_input(cx, PromptInputMode::Enabled, PromptInputButton::Send);
            }
            State::ModelSelectedWithChat {
                receiving_response: true,
                ..
            } => {
                self.activate_prompt_input(
                    cx,
                    PromptInputMode::Disabled,
                    PromptInputButton::EnabledStop,
                );
            }
            State::ModelSelectedWithChat {
                was_cancelled: true,
                ..
            } => {
                self.activate_prompt_input(
                    cx,
                    PromptInputMode::Disabled,
                    PromptInputButton::DisabledStop,
                );
            }
            _ => {
                self.activate_prompt_input(cx, PromptInputMode::Disabled, PromptInputButton::Send);
            }
        }
    }

    fn activate_prompt_input(
        &mut self,
        cx: &mut Cx,
        mode: PromptInputMode,
        button: PromptInputButton,
    ) {
        let prompt = self.command_text_input(id!(main_prompt_input.prompt));
        let prompt_text_input = prompt.text_input_ref();

        let enabled = match mode {
            PromptInputMode::Enabled => !prompt_text_input.text().is_empty(),
            PromptInputMode::Disabled => false,
        };

        let (button_color, prompt_enabled) = if enabled {
            (vec3(0.0, 0.0, 0.0), 1.0)
        } else {
            // The color code is #D0D5DD
            (vec3(0.816, 0.835, 0.867), 0.0)
        };

        prompt_text_input.apply_over(
            cx,
            live! {
                draw_text: { prompt_enabled: (prompt_enabled) }
            },
        );

        let send_button = self
            .prompt_input(id!(main_prompt_input))
            .button(id!(prompt_send_button));
        let stop_button = self
            .prompt_input(id!(main_prompt_input))
            .button(id!(prompt_stop_button));
        match button {
            PromptInputButton::Send => {
                // The send button is enabled or not based on the prompt input
                send_button.set_visible(true);
                send_button.set_enabled(enabled);
                send_button.apply_over(
                    cx,
                    live! {
                        draw_bg: {
                            color: (button_color)
                        }
                    },
                );
                stop_button.set_visible(false);
            }
            PromptInputButton::EnabledStop => {
                stop_button.set_visible(true);
                stop_button.set_enabled(true);
                stop_button.apply_over(
                    cx,
                    live! {
                        draw_bg: {
                            color: #x000
                        }
                    },
                );
                send_button.set_visible(false);
            }
            PromptInputButton::DisabledStop => {
                stop_button.set_visible(true);
                stop_button.set_enabled(false);
                stop_button.apply_over(
                    cx,
                    live! {
                        draw_bg: {
                            color: #D0D5DD
                        }
                    },
                );
                send_button.set_visible(false);
            }
        }
    }

    fn scroll_messages_to_bottom(&mut self, cx: &mut Cx) {
        let list = self.portal_list(id!(chat));
        list.smooth_scroll_to_end(cx, 10., Some(80));
    }

    fn reset_scroll_messages(&mut self, store: &Store) {
        let list = self.portal_list(id!(chat));
        let messages = get_chat_messages(store).unwrap();
        let index = messages.len().saturating_sub(1);
        list.set_first_id(index);
    }

    fn handle_prompt_input_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let prompt = self.command_text_input(id!(main_prompt_input.prompt));
        let prompt_text_input = prompt.text_input_ref();

        if let Some(_text) = prompt_text_input.changed(actions) {
            self.redraw(cx);
        }

        if self
            .button(id!(main_prompt_input.prompt_send_button))
            .clicked(&actions)
        {
            self.send_message(cx, scope, prompt_text_input.text(), None);
        }

        if let Some(prompt) = prompt_text_input.returned(actions) {
            self.send_message(cx, scope, prompt, None);
        }
    }

    fn send_message(
        &mut self,
        cx: &mut Cx,
        scope: &mut Scope,
        prompt: String,
        regenerate_from: Option<usize>,
    ) {
        // Check if we have any text to send
        if prompt.trim().is_empty() {
            return;
        }

        // Let's confirm we're in an appropriate state to send a message
        self.update_state(scope);

        match self.state {
            State::ModelSelectedWithChat {
                receiving_response: false,
                was_cancelled: false,
                is_loading: false,
                ..
            } => {
                self.send_message_aux(cx, scope, prompt, regenerate_from);
            }
            State::ModelSelectedWithEmptyChat { is_loading: false } => {
                self.send_message_aux(cx, scope, prompt, regenerate_from);
                cx.action(ChatAction::TitleUpdated(self.current_chat_id.unwrap()));
            }
            _ => {}
        }
    }

    fn send_message_aux(
        &mut self,
        cx: &mut Cx,
        scope: &mut Scope,
        prompt: String,
        regenerate_from: Option<usize>,
    ) {
        let store = scope.data.get_mut::<Store>().unwrap();

        if let Some(entity_selected) = &self
            .prompt_input(id!(main_prompt_input))
            .borrow()
            .unwrap()
            .entity_selected
        {
            store.send_entity_message(entity_selected, prompt, regenerate_from);
        } else {
            store.send_message_to_current_entity(prompt, regenerate_from);
        }

        self.prompt_input(id!(main_prompt_input)).reset_text(false);

        // Scroll to the bottom when the message is sent
        self.scroll_messages_to_bottom(cx);
        self.redraw(cx);
    }

    fn update_view(&mut self, cx: &mut Cx2d, scope: &mut Scope) {
        self.update_visibilities();
        self.update_prompt_input(cx);

        match self.state {
            State::ModelSelectedWithEmptyChat { .. } => {
                let store = scope.data.get::<Store>().unwrap();

                let chat_entity = &get_chat(store).unwrap().borrow().associated_entity;
                match chat_entity {
                    Some(ChatEntityId::Agent(agent)) => {
                        let empty_view = self.view(id!(empty_conversation));
                        empty_view
                            .view(id!(avatar_section.model))
                            .set_visible(false);
                        empty_view
                            .chat_agent_avatar(id!(avatar_section.agent))
                            .set_visible(true);

                        let agent = store.chats.get_agent_or_placeholder(&agent);
                        empty_view
                            .chat_agent_avatar(id!(avatar_section.agent))
                            .set_agent(agent);
                    }
                    _ => {
                        let empty_view = self.view(id!(empty_conversation));
                        empty_view.view(id!(avatar_section.model)).set_visible(true);
                        empty_view
                            .chat_agent_avatar(id!(avatar_section.agent))
                            .set_visible(false);

                        empty_view
                            .label(id!(avatar_label))
                            .set_text(&get_model_initial_letter(store).unwrap_or('A').to_string());
                    }
                }
            }
            _ => {}
        }
    }

    fn update_visibilities(&mut self) {
        let empty_conversation = self.view(id!(empty_conversation));
        let jump_to_bottom = self.button(id!(jump_to_bottom));
        let main = self.view(id!(main));
        let no_downloaded_model = self.view(id!(no_downloaded_model));
        let no_model = self.view(id!(no_model));

        match self.state {
            // State::NoModelsAvailable => {
            //     empty_conversation.set_visible(false);
            //     jump_to_bottom.set_visible(false);
            //     main.set_visible(false);
            //     no_model.set_visible(false);

            //     no_downloaded_model.set_visible(true);
            // }
            State::NoModelsAvailable | State::NoModelSelected => {
                empty_conversation.set_visible(false);
                jump_to_bottom.set_visible(false);
                main.set_visible(false);
                no_downloaded_model.set_visible(false);

                no_model.set_visible(true);
            }
            State::ModelSelectedWithEmptyChat { .. } => {
                jump_to_bottom.set_visible(false);
                no_downloaded_model.set_visible(false);
                no_model.set_visible(false);

                empty_conversation.set_visible(true);
                main.set_visible(true);
            }
            State::ModelSelectedWithChat {
                sticked_to_bottom, ..
            } => {
                empty_conversation.set_visible(false);
                no_downloaded_model.set_visible(false);
                no_model.set_visible(false);

                main.set_visible(true);
                jump_to_bottom.set_visible(!sticked_to_bottom);
            }
            _ => {}
        }
    }

    fn draw_messages(&mut self, cx: &mut Cx2d, scope: &mut Scope, list: &mut RefMut<PortalList>) {
        let store = scope.data.get::<Store>().unwrap();
        let messages = get_chat_messages(store).unwrap();
        let messages_count = messages.len();

        self.portal_list_end_reached = false;
        list.set_item_range(cx, 0, messages_count + 1);
        while let Some(item_id) = list.next_visible_item(cx) {
            if item_id < messages_count {
                let chat_line_data = &messages[item_id];

                let item;
                let mut chat_line_item;
                if chat_line_data.is_assistant() {
                    item = list.item(cx, item_id, live_id!(ModelChatLine));
                    chat_line_item = item.as_chat_line();

                    let username = chat_line_data.username.as_ref().map_or("", String::as_str);
                    chat_line_item.set_sender_name(&username);
                    chat_line_item.set_regenerate_button_visible(false);

                    match &chat_line_data.entity {
                        Some(ChatEntityId::Agent(agent_id)) => {
                            let agent = store.chats.get_agent_or_placeholder(&agent_id);
                            chat_line_item.set_model_avatar(agent);
                        }
                        Some(ChatEntityId::ModelFile(_)) => {
                            chat_line_item.set_model_avatar_text(
                                &get_model_initial_letter(store).unwrap().to_string(),
                            );
                        }
                        _ => {}
                    }
                } else {
                    item = list.item(cx, item_id, live_id!(UserChatLine));
                    chat_line_item = item.as_chat_line();
                    chat_line_item.set_regenerate_button_visible(true);
                };

                chat_line_item.set_message_id(chat_line_data.id);

                // Disable actions for the last chat line when model is streaming
                if matches!(
                    self.state,
                    State::ModelSelectedWithChat {
                        receiving_response: true,
                        ..
                    }
                ) && item_id == messages_count - 1
                {
                    chat_line_item.set_message_text(cx, &chat_line_data.content, true);
                    chat_line_item.set_actions_enabled(cx, false);
                } else {
                    chat_line_item.set_message_text(cx, &chat_line_data.content, false);
                    chat_line_item.set_actions_enabled(cx, true);
                }

                item.draw_all(cx, &mut Scope::empty());
            } else {
                self.portal_list_end_reached = true;
                let item = list.item(cx, item_id, live_id!(EndOfChat));
                item.draw_all(cx, &mut Scope::empty());
            }
        }
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ChatPanelAction {
    NavigateToDiscover,
    None,
}

fn get_chat(store: &Store) -> Option<&RefCell<Chat>> {
    store.chats.get_current_chat()
}

fn get_model_initial_letter(store: &Store) -> Option<char> {
    let chat = get_chat(store)?;
    let initial_letter = store
        .get_chat_entity_name(chat.borrow().id)
        .map(|name| name.chars().next())?;

    initial_letter.map(|letter| letter.to_ascii_uppercase())
}

fn get_chat_messages(store: &Store) -> Option<Ref<Vec<ChatMessage>>> {
    get_chat(store).map(|chat| Ref::map(chat.borrow(), |chat| &chat.messages))
}

fn get_chat_id(store: &Store) -> Option<ChatID> {
    get_chat(store).map(|chat| chat.borrow().id)
}
