use makepad_widgets::*;
use moxin_protocol::data::FileID;
use std::cell::{Ref, RefCell, RefMut};

use crate::{
    chat::{
        chat_line::{ChatLineAction, ChatLineWidgetRefExt},
        model_selector::ModelSelectorWidgetExt,
        model_selector_list::ModelSelectorAction,
    },
    data::{
        chats::chat::{Chat, ChatMessage},
        store::Store,
    },
    shared::actions::ChatAction,
};

use super::chat_history_card::ChatHistoryCardAction;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import makepad_draw::shader::std::*;

    import crate::chat::model_selector::ModelSelector;
    import crate::chat::chat_line::ChatLine;
    import crate::chat::shared::ChatAgentAvatar;

    ICON_PROMPT = dep("crate://self/resources/icons/prompt.svg")
    ICON_STOP = dep("crate://self/resources/icons/stop.svg")
    ICON_JUMP_TO_BOTTOM = dep("crate://self/resources/icons/jump_to_bottom.svg")

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
                            draw_normal: {
                                color: (#fff),
                            }
                            draw_italic: {
                                color: (#fff),
                            }
                            draw_bold: {
                                color: (#fff),
                            }
                            draw_bold_italic: {
                                color: (#fff),
                            }
                            draw_fixed: {
                                color: (#fff),
                            }
                            draw_block: {
                                line_color: (#fff)
                                sep_color: (#12778a)
                                quote_bg_color: (#12778a)
                                quote_fg_color: (#106a7b)
                                block_color: (#12778a)
                                code_color: (#12778a)
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
                    input_container = {
                        input = {
                            draw_bg: {
                                color: #15859A
                            }
                            draw_text: {
                                fn get_color(self) -> vec4 {
                                    return #fff;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    ModelChatLine = <ChatLine> {
        avatar_section = {
            <ChatAgentAvatar> {}
        }
        main_section = {
            body_section = {
                bubble = {
                    draw_bg: {
                        border_width: 1.0,
                        border_color: #D0D5DD,
                    }
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
                                block_color: (#EDEDED)
                                code_color: (#EDEDED)
                            }
                        }
                    }
                }
            }
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
            margin: {bottom: 10},

            cursor: Hand,

            show_bg: true,

            draw_bg: {
                radius: 14.0,
                color: #fff,
                border_width: 1.0,
                border_color: #EAECF0,
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
            radius: 10.0,
            border_color: #D0D5DD,
            border_width: 1.0,
        }

        prompt = <MoxinTextInput> {
            width: Fill,
            height: Fit,

            empty_message: "Enter a message"
            draw_bg: {
                radius: 30.0
                color: #fff
            }
            draw_text: {
                text_style:<REGULAR_FONT>{font_size: 10},

                instance prompt_enabled: 0.0
                fn get_color(self) -> vec4 {
                    return mix(
                        #98A2B3,
                        #000,
                        self.prompt_enabled
                    )
                }
            }
        }

        prompt_icon = <RoundedView> {
            width: 28,
            height: 28,
            show_bg: true,
            draw_bg: {
                radius: 7.0,
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

        no_downloaded_model = <View> {
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
                go_to_discover_button = <MoxinButton> {
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
                no_downloaded_model_prompt_input = <ChatPromptInput> {}
            }

        }

        no_model = <View> {
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
                no_model_prompt_input = <ChatPromptInput> {}
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
                    scroll_bar: {
                        bar_size: 0.0,
                    }
                    width: Fill,
                    height: Fill,

                    drag_scrolling: false,

                    UserChatLine = <UserChatLine> {}
                    ModelChatLine = <ModelChatLine> {}
                }

                <JumpToButtom> {}
            }

            main_prompt_input = <ChatPromptInput> {}
        }

        model_selector = <ModelSelector> {}
    }
}

#[derive(Clone, Copy, Debug, Default)]
enum State {
    /// `Unknown` is simply the default state, meaning the state has not been loaded yet,
    /// and therefore indicates a development error if it is encountered.
    #[default]
    Unknown,
    NoModelsAvailable,
    NoModelSelected,
    ModelSelectedWithEmptyChat,
    ModelSelectedWithChat {
        sticked_to_bottom: bool,
        is_streaming: bool,
    },
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatPanel {
    #[deref]
    view: View,

    #[rust]
    state: State,
}

impl Widget for ChatPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
        self.update_state(scope);

        if let Event::Signal = event {
            match self.state {
                State::ModelSelectedWithChat {
                    is_streaming: true,
                    sticked_to_bottom,
                } => {
                    if sticked_to_bottom {
                        self.scroll_messages_to_bottom(cx);
                    }

                    // Redraw because we expect to see new or updated chat entries
                    self.redraw(cx);
                }
                State::NoModelSelected | State::NoModelsAvailable => {
                    self.model_selector(id!(model_selector)).deselect(cx);
                    self.view.redraw(cx);
                }
                _ => {}
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.update_view(cx, scope);

        while let Some(view_item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = view_item.as_portal_list().borrow_mut() {
                self.draw_messages(cx, scope, &mut list);
            }
        }

        DrawStep::done()
    }
}

impl WidgetMatchEvent for ChatPanel {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let widget_uid = self.widget_uid();
        let store = scope.data.get_mut::<Store>().unwrap();

        for action in actions
            .iter()
            .filter_map(|action| action.as_widget_action())
        {
            if let ChatHistoryCardAction::ChatSelected(_) = action.cast() {
                self.redraw(cx);
            }

            if let ModelSelectorAction::Selected(downloaded_file) = action.cast() {
                store.load_model(&downloaded_file.file);
                self.redraw(cx)
            }

            match action.cast() {
                ChatAction::Start(file_id) => {
                    store.ensure_model_loaded(file_id);
                    store.chats.create_empty_chat();
                }
                ChatAction::Resume(file_id) => {
                    store.ensure_model_loaded(file_id);
                }
                _ => {}
            }

            match action.cast() {
                ChatLineAction::Delete(id) => {
                    store.chats.delete_chat_message(id);
                    self.redraw(cx);
                }
                ChatLineAction::Edit(id, updated, regenerate) => {
                    store.chats.edit_chat_message(id, updated, regenerate);
                    self.redraw(cx);
                }
                _ => {}
            }
        }

        if let Some(fe) = self.view(id!(jump_to_bottom)).finger_up(actions) {
            if fe.was_tap() {
                self.scroll_messages_to_bottom(cx);
                self.redraw(cx);
            }
        }

        match self.state {
            State::ModelSelectedWithChat {
                is_streaming: false,
                ..
            }
            | State::ModelSelectedWithEmptyChat => {
                self.handle_prompt_input_actions(cx, actions, scope);
            }
            State::ModelSelectedWithChat {
                is_streaming: true, ..
            } => {
                if let Some(fe) = self
                    .view(id!(main_prompt_input.prompt_icon))
                    .finger_up(actions)
                {
                    if fe.was_tap() {
                        store.chats.cancel_chat_streaming();
                    }
                }
            }
            _ => {}
        }

        if self
            .button(id!(no_downloaded_model.go_to_discover_button))
            .clicked(actions)
        {
            cx.widget_action(widget_uid, &scope.path, ChatPanelAction::NavigateToDiscover);
        }
    }
}

impl ChatPanel {
    fn update_state(&mut self, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();
        let list = self.portal_list(id!(chat));

        self.state = if store.downloads.downloaded_files.is_empty() {
            State::NoModelsAvailable
        } else if store.chats.loaded_model.is_none() {
            State::NoModelSelected
        } else {
            store
                .chats
                .get_current_chat()
                .map_or(State::ModelSelectedWithEmptyChat, |chat| {
                    if chat.borrow().messages.is_empty() {
                        State::ModelSelectedWithEmptyChat
                    } else {
                        State::ModelSelectedWithChat {
                            sticked_to_bottom: !list.further_items_bellow_exist()
                                || !matches!(self.state, State::ModelSelectedWithChat { .. }),
                            is_streaming: chat.borrow().is_streaming,
                        }
                    }
                })
        };
    }

    fn update_prompt_input(&mut self, cx: &mut Cx) {
        match self.state {
            State::ModelSelectedWithEmptyChat
            | State::ModelSelectedWithChat {
                is_streaming: false,
                ..
            } => {
                self.enable_or_disable_prompt_input(cx);
                self.show_prompt_input_send_icon(cx);
            }
            State::ModelSelectedWithChat {
                is_streaming: true, ..
            } => {
                let prompt_input = self.text_input(id!(main_prompt_input.prompt));
                prompt_input.apply_over(
                    cx,
                    live! {
                        draw_text: { prompt_enabled: 0.0 }
                    },
                );
                self.show_prompt_input_stop_icon(cx);
            }
            _ => {}
        }
    }

    fn enable_or_disable_prompt_input(&mut self, cx: &mut Cx) {
        let prompt_input = self.text_input(id!(main_prompt_input.prompt));
        let enable = if !prompt_input.text().is_empty() {
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
        self.view(id!(main_prompt_input.prompt_icon)).apply_over(
            cx,
            live! {
                icon_send = { visible: true }
                icon_stop = { visible: false }
            },
        );
        let prompt_input = self.text_input(id!(main_prompt_input.prompt));
        if !prompt_input.text().is_empty() {
            self.enable_prompt_input_icon(cx);
        } else {
            self.disable_prompt_input_icon(cx);
        }
    }

    fn show_prompt_input_stop_icon(&mut self, cx: &mut Cx) {
        self.view(id!(main_prompt_input.prompt_icon)).apply_over(
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
        self.view(id!(main_prompt_input.prompt_icon)).apply_over(
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
        self.view(id!(main_prompt_input.prompt_icon)).apply_over(
            cx,
            live! {
                draw_bg: {
                    color: (disabled_color)
                }
            },
        );
    }

    fn scroll_messages_to_bottom(&mut self, cx: &mut Cx) {
        let mut list = self.portal_list(id!(chat));
        list.smooth_scroll_to_end(cx, 10, 80.0);
    }

    fn handle_prompt_input_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let prompt_input = self.text_input(id!(main_prompt_input.prompt));
        if let Some(_text) = prompt_input.changed(actions) {
            self.update_prompt_input(cx);
        }

        if let Some(fe) = self
            .view(id!(main_prompt_input.prompt_icon))
            .finger_up(&actions)
        {
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

        let store = scope.data.get_mut::<Store>().unwrap();
        store.chats.send_chat_message(prompt.clone());

        let prompt_input = self.text_input(id!(main_prompt_input.prompt));
        prompt_input.set_text_and_redraw(cx, "");
        prompt_input.set_cursor(0, 0);
        self.update_prompt_input(cx);

        // Scroll to the bottom when the message is sent
        self.scroll_messages_to_bottom(cx);
    }

    fn update_view(&mut self, cx: &mut Cx2d, scope: &mut Scope) {
        self.update_visibilities();
        self.update_prompt_input(cx);

        match self.state {
            State::ModelSelectedWithEmptyChat => {
                let store = scope.data.get::<Store>().unwrap();

                self.view(id!(empty_conversation))
                    .label(id!(avatar_label))
                    .set_text(&get_model_initial_letter(store).unwrap_or_default().to_string());
            }
            _ => {}
        }
    }

    fn update_visibilities(&mut self) {
        let empty_conversation = self.view(id!(empty_conversation));
        let jump_to_bottom = self.view(id!(jump_to_bottom));
        let main = self.view(id!(main));
        let no_downloaded_model = self.view(id!(no_downloaded_model));
        let no_model = self.view(id!(no_model));

        match self.state {
            State::NoModelsAvailable => {
                empty_conversation.set_visible(false);
                jump_to_bottom.set_visible(false);
                main.set_visible(false);
                no_model.set_visible(false);

                no_downloaded_model.set_visible(true);
            }
            State::NoModelSelected => {
                empty_conversation.set_visible(false);
                jump_to_bottom.set_visible(false);
                main.set_visible(false);
                no_downloaded_model.set_visible(false);

                no_model.set_visible(true);
            }
            State::ModelSelectedWithEmptyChat => {
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

    fn draw_messages(&self, cx: &mut Cx2d, scope: &mut Scope, list: &mut RefMut<PortalList>) {
        let store = scope.data.get::<Store>().unwrap();
        let Some(messages) = get_chat_messages(store) else { return };
        let messages_count = messages.len();

        list.set_item_range(cx, 0, messages_count);
        while let Some(item_id) = list.next_visible_item(cx) {
            if item_id < messages_count {
                let chat_line_data = &messages[item_id];

                let item;
                let mut chat_line_item;
                if chat_line_data.is_assistant() {
                    item = list.item(cx, item_id, live_id!(ModelChatLine)).unwrap();
                    chat_line_item = item.as_chat_line();

                    let username = chat_line_data.username.as_ref().map_or("", String::as_str);
                    chat_line_item.set_sender_name(&username);
                    chat_line_item.set_regenerate_enabled(false);
                    chat_line_item
                        .set_avatar_text(&get_initial_letter(username).unwrap().to_string());
                } else {
                    item = list.item(cx, item_id, live_id!(UserChatLine)).unwrap();
                    chat_line_item = item.as_chat_line();
                    chat_line_item.set_regenerate_enabled(true);
                };

                chat_line_item.set_message_text(cx, &chat_line_data.content);
                chat_line_item.set_message_id(chat_line_data.id);

                // Disable actions for the last chat line when model is streaming
                if matches!(
                    self.state,
                    State::ModelSelectedWithChat {
                        is_streaming: true,
                        ..
                    }
                ) && item_id == messages_count - 1
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

#[derive(Clone, DefaultNone, Debug)]
pub enum ChatPanelAction {
    UnloadIfActive(FileID),
    NavigateToDiscover,
    None,
}

fn get_chat(store: &Store) -> Option<&RefCell<Chat>> {
    store.chats.get_current_chat()
}

fn get_initial_letter(word: &str) -> Option<char> {
    word.chars().next()
}

fn get_model_initial_letter(store: &Store) -> Option<char> {
    let chat = get_chat(store)?;
    let initial_letter = get_initial_letter(&chat.borrow().model_filename)?;
    Some(initial_letter.to_ascii_uppercase())
}

fn get_chat_messages(store: &Store) -> Option<Ref<Vec<ChatMessage>>> {
    get_chat(store).map(|chat| Ref::map(chat.borrow(), |chat| &chat.messages))
}
