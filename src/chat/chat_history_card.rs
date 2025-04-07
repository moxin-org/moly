use crate::{
    data::{
        chats::chat::ChatID,
        store::Store,
    },
    shared::{actions::ChatAction, modal::ModalWidgetExt, utils::human_readable_name},
};

use makepad_widgets::*;

use super::{
    chat_history_card_options::ChatHistoryCardOptionsWidgetExt,
    delete_chat_modal::DeleteChatModalAction,
};
use super::{delete_chat_modal::DeleteChatModalWidgetExt, shared::ChatAgentAvatarWidgetExt};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::shared::modal::*;
    use crate::chat::shared::ChatAgentAvatar;
    use crate::chat::shared::ChatModelAvatar;
    use crate::chat::chat_history_card_options::ChatHistoryCardOptions;
    use crate::chat::delete_chat_modal::DeleteChatModal;

    ICON_DELETE = dep("crate://self/resources/icons/delete.svg")

    EditTextInput = <MolyTextInput> {
        width: Fill,
        height: Fit,
        padding: 6,
        empty_message: ""

        draw_text: {
            text_style:<REGULAR_FONT>{font_size: 10},
            word: Wrap,

            instance prompt_enabled: 0.0
            fn get_color(self) -> vec4 {
                return #000;
            }
        }
    }

    EditActionButton = <MolyButton> {
        width: 56,
        height: 31,
        spacing: 6,

        draw_bg: { color: #099250 }

        draw_text: {
            text_style: <REGULAR_FONT>{font_size: 9},
            fn get_color(self) -> vec4 {
                return #fff;
            }
        }
    }

    SaveButton = <EditActionButton> {
        text: "Save"
    }

    CancelButton = <EditActionButton> {
        draw_bg: { border_color_1: #D0D5DD, border_size: 1.0, color: #fff }

        draw_text: {
            text_style: <REGULAR_FONT>{font_size: 9},
            fn get_color(self) -> vec4 {
                return #000;
            }
        }
        text: "Cancel"
    }

    pub ChatHistoryCard = {{ChatHistoryCard}} {
        flow: Overlay,
        width: Fill,
        height: 52,

        selected_bg = <RoundedInnerShadowView> {
            width: Fill
            height: Fill
            padding: {left: 4, right: 4}

            show_bg: true

            draw_bg: {
                border_radius: 5.0,
                shadow_color: #47546722
                shadow_radius: 25.0
                shadow_offset: vec2(-2.0, 1.0)
                border_color: #D0D5DD
            }
        }

        content = <RoundedView> {
            width: Fill
            height: Fit
            flow: Right
            padding: {left: 4, right: 4}
            spacing: 4

            cursor: Hand
            show_bg: true
            draw_bg: {
                instance down: 0.0,
                color: #0000
                border_size: 0
                border_radius: 5
            }

            <View> {
                width: Fit
                height: 50
                align: {y: 0.5}
                padding: {left: 4}

                avatar_section = <View> {
                    width: Fit, height: Fit,
                    model = <ChatModelAvatar> {}
                    agent = <ChatAgentAvatar> { visible: false }
                }
            }
            <View> {
                width: Fill
                height: Fit
                flow: Down
                align: {y: 0.5}
                spacing: 2
                padding: { left: 8, top: 10, bottom: 10 }

                model_or_agent_name_label = <Label> {
                    width: Fill,
                    height: Fit,
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 7.5},
                        color: #475467,
                    }
                }

                <View> {
                    width: Fill
                    height: Fit
                    flow: Right
                    spacing: 10
                    padding: { top: 4, bottom: 4 }

                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Down,

                        title_input_container = <View> {
                            visible: false,
                            width: Fill,
                            height: Fit,
                            title_input = <EditTextInput> {}
                        }

                        title_label_container = <View> {
                            visible: false,
                            width: Fill,
                            height: Fit,

                            title_label = <Label> {
                                width: Fill,
                                height: Fit,
                                draw_text: {
                                    text_style: <REGULAR_FONT>{font_size: 9.5},
                                    color: #101828,
                                }
                                text: ""
                            }
                        }

                        edit_buttons = <View> {
                            visible: false,
                            width: Fit,
                            height: Fit,
                            margin: {top: 10},
                            spacing: 6,
                            save = <SaveButton> {}
                            cancel = <CancelButton> {}
                        }
                    }
                }
            }

            chat_options_wrapper = <View> {
                width: Fit
                height: Fill
                padding: 4

                chat_options = <MolyButton> {
                    width: Fit
                    height: Fit
                    margin: {top: 10}
                    padding: { top: 0, right: 4, bottom: 6, left: 4 }

                    draw_bg: {
                        border_radius: 5
                    }

                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 12},
                        color: #667085,
                    }
                    text: "..."

                    reset_hover_on_click: false
                }
            }
            animator: {
                hover = {
                    default: off
                    off = {
                        from: {all: Forward {duration: 0.15}}
                        apply: {
                            draw_bg: {color: #F2F4F700}
                        }
                    }
                    on = {
                        from: {all: Forward {duration: 0.15}}
                        apply: {
                            draw_bg: {color: #EAECEF88}
                        }
                    }
                }
                down = {
                    default: off
                    off = {
                        from: {all: Forward {duration: 0.5}}
                        ease: OutExp
                        apply: {
                            draw_bg: {instance down: 0.0}
                        }
                    }
                    on = {
                        ease: OutExp
                        from: {
                            all: Forward {duration: 0.2}
                        }
                        apply: {
                            draw_bg: {instance down: 1.0}
                        }
                    }
                }
            }
        }

        chat_history_card_options_modal = <Modal> {
            align: {x: 0.0, y: 0.0}
            bg_view: {
                visible: false
            }
            content: {
                chat_history_card_options = <ChatHistoryCardOptions> {}
            }
        }

        delete_chat_modal = <Modal> {
            content: {
                delete_chat_modal_inner = <DeleteChatModal> {}
            }
        }

        animator: {
            edit = {
                default: hide
                show = {
                    from: {all: Forward {duration: 0.15}}
                    apply: {
                        height: 108
                    }
                }
                hide = {
                    from: {all: Forward {duration: 0.15}}
                    apply: {
                        height: 52
                    }
                }
            }
        }
    }
}

#[derive(Default, Debug, PartialEq)]
enum TitleState {
    OnEdit,
    #[default]
    Editable,
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatHistoryCard {
    #[deref]
    view: View,
    #[rust]
    chat_id: ChatID,

    #[rust]
    title_edition_state: TitleState,

    #[animator]
    animator: Animator,
}

impl Widget for ChatHistoryCard {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>().unwrap();
        let chat = store
            .chats
            .saved_chats
            .iter()
            .find(|c| c.borrow().id == self.chat_id)
            .unwrap();

        if let Some(current_chat_id) = store.chats.get_current_chat_id() {
            let content_view_highlight = self.view(id!(selected_bg));

            if current_chat_id == self.chat_id {
                content_view_highlight.apply_over(
                    cx,
                    live! {
                        draw_bg: {color: #ebedee}
                    },
                );
            } else {
                content_view_highlight.apply_over(
                    cx,
                    live! {
                        draw_bg: {color: #x0000}
                    },
                );
            }
        }

        let caption = store.get_chat_associated_bot(self.chat_id)
            .map(|bot_id| store.chats.available_bots.get(&bot_id).map(|m| m.name.clone()).unwrap_or("Unknown".to_string()));
        self.set_title_text(
            cx,
            chat.borrow_mut().get_title(),
            &caption.clone().unwrap_or_default(),
        );
        self.update_title_visibility(cx);

        let initial_letter = caption
            .unwrap_or("a".to_string())
            .chars()
            .next()
            .unwrap()
            .to_lowercase()
            .to_string();

        match &chat.borrow().associated_bot {
            Some(bot_id) => {
                let bot = store.chats.get_bot_or_placeholder(&bot_id);

                if store.chats.is_agent(bot_id) { 
                    let mut chat_agent_avatar = self.chat_agent_avatar(id!(avatar_section.agent));
                    self.view(id!(avatar_section.model)).set_visible(cx, false);
                    chat_agent_avatar.set_visible(true);
                    chat_agent_avatar.set_bot(bot);
                } else {
                    self.view(id!(avatar_section.model)).set_visible(cx, true);
                    self.chat_agent_avatar(id!(avatar_section.agent))
                        .set_visible(false);
                    self.view.label(id!(avatar_label)).set_text(cx, &initial_letter);
                }
            }
            _ => {}
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ChatHistoryCard {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        // let widget_uid = self.widget_uid();

        match self.title_edition_state {
            TitleState::Editable => self.handle_title_editable_actions(cx, actions, scope),
            TitleState::OnEdit => self.handle_title_on_edit_actions(cx, actions, scope),
        }

        let chat_options_wrapper_rect = self.view(id!(chat_options_wrapper)).area().rect(cx);
        if self.button(id!(chat_options)).clicked(actions) {
            let wrapper_coords = chat_options_wrapper_rect.pos;
            let coords = dvec2(
                wrapper_coords.x,
                wrapper_coords.y + chat_options_wrapper_rect.size.y - 12.0,
            );

            self.chat_history_card_options(id!(chat_history_card_options))
                .selected(cx, self.chat_id);

            let modal = self.modal(id!(chat_history_card_options_modal));
            modal.apply_over(
                cx,
                live! {
                    content: { margin: { left: (coords.x), top: (coords.y) } }
                },
            );
            modal.open(cx);
            return;
        }

        if let Some(fe) = self.view(id!(content)).finger_down(actions) {
            if fe.tap_count == 1 {
                let store = scope.data.get_mut::<Store>().unwrap();
                store.chats.set_current_chat(Some(self.chat_id));
                cx.action(ChatAction::ChatSelected(self.chat_id));
                self.redraw(cx);
            }
        }

        for action in actions {
            if matches!(
                action.cast(),
                DeleteChatModalAction::Cancelled
                    | DeleteChatModalAction::CloseButtonClicked
                    | DeleteChatModalAction::ChatDeleted
            ) {
                self.modal(id!(delete_chat_modal)).close(cx);
            }
        }
    }
}

impl ChatHistoryCard {
    pub fn set_chat_id(&mut self, id: ChatID) {
        if id != self.chat_id {
            self.chat_id = id;
            self.title_edition_state = TitleState::Editable;
        }
    }

    fn set_title_text(&mut self, cx: &mut Cx, text: &str, caption: &str) {
        self.view.label(id!(title_label)).set_text(cx, text.trim());
        if let TitleState::Editable = self.title_edition_state {
            self.view.text_input(id!(title_input)).set_text(cx, text.trim());
        }
        self.label(id!(model_or_agent_name_label))
            .set_text(cx, &human_readable_name(caption));
    }

    fn update_title_visibility(&mut self, cx: &mut Cx) {
        let on_edit = matches!(self.title_edition_state, TitleState::OnEdit);
        self.view(id!(edit_buttons)).set_visible(cx, on_edit);
        self.view(id!(title_input_container)).set_visible(cx, on_edit);
        self.button(id!(chat_options)).set_visible(cx, !on_edit);

        let editable = matches!(self.title_edition_state, TitleState::Editable);
        self.view(id!(title_label_container)).set_visible(cx, editable);
    }

    fn transition_title_state(&mut self, cx: &mut Cx) {
        self.title_edition_state = match self.title_edition_state {
            TitleState::OnEdit => TitleState::Editable,
            TitleState::Editable => TitleState::OnEdit,
        };

        self.update_title_visibility(cx);
        self.redraw(cx);

        match self.title_edition_state {
            TitleState::OnEdit => {
                self.animator_play(cx, id!(edit.show));
            }
            TitleState::Editable => {
                self.animator_play(cx, id!(edit.hide));
            }
        }
    }

    pub fn handle_title_editable_actions(
        &mut self,
        cx: &mut Cx,
        actions: &Actions,
        _scope: &mut Scope,
    ) {
        for action in actions {
            match action.cast() {
                ChatHistoryCardAction::MenuClosed(chat_id) => {
                    if chat_id == self.chat_id {
                        self.button(id!(chat_options)).reset_hover(cx);
                        self.modal(id!(chat_history_card_options_modal)).close(cx);
                    }
                }
                ChatHistoryCardAction::ActivateTitleEdition(chat_id) => {
                    if chat_id == self.chat_id {
                        self.transition_title_state(cx);
                    }
                }
                ChatHistoryCardAction::DeleteChatOptionSelected(chat_id) => {
                    if chat_id == self.chat_id {
                        let mut delete_modal_inner =
                            self.delete_chat_modal(id!(delete_chat_modal_inner));
                        delete_modal_inner.set_chat_id(self.chat_id);

                        self.modal(id!(delete_chat_modal)).open(cx);
                    }
                }
                _ => {}
            }

            // If the modal is dissmised (such as, clicking outside) we need to reset the hover state
            // of the open chat options button.
            if self
                .modal(id!(chat_history_card_options_modal))
                .dismissed(actions)
            {
                self.button(id!(chat_options)).reset_hover(cx);
            }
        }
    }

    fn handle_title_on_edit_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        if self.button(id!(save)).clicked(actions) {
            let updated_title = self.text_input(id!(title_input)).text();
            let chat = store
                .chats
                .saved_chats
                .iter()
                .find(|c| c.borrow().id == self.chat_id)
                .unwrap();

            if !updated_title.trim().is_empty() && chat.borrow().get_title() != updated_title {
                chat.borrow_mut().set_title(updated_title.clone());
                chat.borrow().save();
            }

            self.transition_title_state(cx)
        }

        if let Some(val) = self.text_input(id!(title_input)).returned(actions) {
            let chat = store
                .chats
                .saved_chats
                .iter()
                .find(|c| c.borrow().id == self.chat_id)
                .unwrap();

            if !val.trim().is_empty() && chat.borrow().get_title() != val {
                chat.borrow_mut().set_title(val.clone());
                chat.borrow().save();
            }

            self.transition_title_state(cx)
        }

        if self.button(id!(cancel)).clicked(actions) {
            self.transition_title_state(cx)
        }
    }
}

impl ChatHistoryCardRef {
    pub fn set_chat_id(&mut self, id: ChatID) -> Result<(), &'static str> {
        let Some(mut inner) = self.borrow_mut() else {
            return Err("Widget not found in the document");
        };

        inner.set_chat_id(id);
        Ok(())
    }
}

#[derive(Clone, DefaultNone, Eq, Hash, PartialEq, Debug)]
pub enum ChatHistoryCardAction {
    None,
    ActivateTitleEdition(ChatID),
    MenuClosed(ChatID),
    DeleteChatOptionSelected(ChatID),
}
