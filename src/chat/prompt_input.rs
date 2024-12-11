use makepad_widgets::*;

use crate::{
    data::{
        chats::chat_entity::{ChatEntityId, ChatEntityRef},
        store::Store,
    },
    shared::{actions::ChatAction, list::ListWidgetExt},
};

use super::{
    entity_button::EntityButtonWidgetRefExt, model_selector_item::ModelSelectorAction,
    shared::ChatAgentAvatarWidgetExt,
};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::shared::list::*;
    use crate::chat::entity_button::*;
    use crate::chat::shared::ChatAgentAvatar;

    ICON_PROMPT = dep("crate://self/resources/icons/prompt.svg")
    ICON_STOP = dep("crate://self/resources/icons/stop.svg")

    CircleButton = <MolyButton> {
        padding: {right: 2},
        margin: {bottom: 2},

        draw_icon: {
            color: #fff
        }
        icon_walk: {width: 12, height: 12}
    }

    PromptButton = <CircleButton> {
        width: 28,
        height: 28,

        draw_bg: {
            radius: 6.5,
            color: #D0D5DD
        }
        icon_walk: {
            margin: {top: 0, left: -4},
        }
    }

    pub PromptInput = {{PromptInput}} {
        flow: Overlay,
        height: Fit,
        entity_template: <View> {
            width: Fill,
            height: Fit,
            show_bg: true,

            button = <EntityButton> {
                server_url_visible: true,
            }
        }
        section_label_template: <Label> {
            padding: {top: 4., bottom: 4.}
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 10.0},
                color: #98A2B3
            }
        }

        <View> {
            flow: Down,
            height: Fit,
            autocomplete = <View> {
                height: Fit,
                visible: false,
                align: {x: 0.5, y: 1.0},
                margin: {bottom: 10},
                <RoundedView> {
                    flow: Down,
                    height: Fit,
                    padding: {bottom: 12.0, top: 12.0, right: 6.0, left: 6.0}
                    show_bg: true,
                    draw_bg: {
                        border_width: 1.0,
                        border_color: #D0D5DD,
                        color: #fff,
                        radius: 5.0
                    }

                    search_input = <MolyTextInput> {
                        width: Fill,
                        height: Fit,
                        margin: {bottom: 4},
                        empty_message: "Search for a model or agent",
                        draw_bg: {
                            radius: 5.0,
                            color: #fff
                        }
                        draw_text: {
                            text_style: <REGULAR_FONT>{font_size: 10},
                            color: #475467
                        }
                    }

                    list = <List> {
                        padding: {left: 4.}
                        height: Fit 
                    }
                }
            }

            <RoundedView> {
                flow: Down,
                width: Fill,
                height: Fit,
                padding: {top: 8, bottom: 6, left: 4, right: 10}
                spacing: 4,
                align: {x: 0.0, y: 1.0},

                show_bg: true,
                draw_bg: {
                    color: #fff,
                    radius: 10.0,
                    border_color: #D0D5DD,
                    border_width: 1.0,
                }

                selected_bubble = <RoundedView> {
                    visible: false,
                    flow: Right,
                    width: Fit,
                    height: Fit,
                    align: {y: 0.5},
                    margin: 5.0
                    padding: {left: 10, right: 10, top: 8, bottom: 8}
                    show_bg: true,
                    draw_bg: {
                        color: #F2F4F7,
                        radius: 10.0,
                    }
                    agent_avatar = <ChatAgentAvatar> {
                        width: Fit,
                        height: Fit,
                        image = {
                            width: 20, height: 20, margin: {right: 8}
                        }
                    }
                    <Label> {
                        text: "Chat with "
                        draw_text: {
                            text_style: <REGULAR_FONT>{font_size: 8},
                            color: #475467
                        }
                    }
                    selected_label = <Label> {
                        margin: {right: 4},
                        draw_text: {
                            text_style: <BOLD_FONT>{font_size: 8},
                            color: #000
                        }
                    }
                    deselect_button = <MolyButton> {
                        width: 8,
                        height: 8,
                        padding: 0,
                        draw_bg: {
                            color: #00000000,
                            color_hover: #00000000,
                            border_color_hover: #00000000,
                        }
                        icon_walk: {width: 8, height: 8}
                        draw_icon: {
                            svg_file: dep("crate://self/resources/icons/close.svg"),
                            color: #475467
                        }
                    }
                }

                <View> {
                    flow: Right,
                    width: Fill,
                    height: Fit,
                    prompt = <MolyTextInput> {
                        width: Fill,
                        height: Fit,
                        draw_bg: {
                            radius: 10.0
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

                    prompt_send_button = <PromptButton> {
                        draw_icon: {
                            svg_file: (ICON_PROMPT),
                        }
                    }

                    prompt_stop_button = <PromptButton> {
                        visible: false,
                        draw_icon: {
                            svg_file: (ICON_STOP),
                        }
                    }
                }
            }
        }

    }
}

#[derive(Widget, Live)]
pub struct PromptInput {
    #[deref]
    deref: View,

    #[live]
    entity_template: Option<LivePtr>,

    #[live]
    section_label_template: Option<LivePtr>,

    // see `was_at_added` function
    #[rust]
    prev_prompt: String,

    /// The index of the currently focused item in the autocomplete list.
    /// Starts at 1 to account for the section labels
    #[rust(1usize)]
    keyboard_focus_index: usize,

    #[rust]
    search_pending_focus: bool,

    #[rust]
    prompt_pending_focus: bool,

    #[rust]
    pub entity_selected: Option<ChatEntityId>,
}

impl Widget for PromptInput {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while !self.deref.draw_walk(cx, scope, walk).is_done() {}

        if self.search_pending_focus {
            self.search_pending_focus = false;

            let search_input = self.text_input(id!(search_input));
            set_cursor_to_end(&search_input);
            search_input.set_key_focus(cx);
        }

        if self.prompt_pending_focus {
            self.prompt_pending_focus = false;

            let prompt = self.text_input(id!(prompt));
            set_cursor_to_end(&prompt);
            prompt.set_key_focus(cx);
        }

        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);

        // since we are hiding this on blur, checking visibility is enough to know if it is focused
        if self.view(id!(autocomplete)).visible() {
            if let Event::KeyDown(key_event) = event {
                let delta = match key_event.key_code {
                    KeyCode::ArrowDown => 1,
                    KeyCode::ArrowUp => -1,
                    _ => 0,
                };

                if delta != 0 {
                    self.on_search_keyboard_move(cx, scope, delta);
                }
            }
        }

        self.widget_match_event(cx, event, scope);
    }
}

impl WidgetMatchEvent for PromptInput {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let prompt = self.text_input(id!(prompt));
        let search_input = self.text_input(id!(search_input));

        let clicked_entity_button = self.list(id!(autocomplete.list)).borrow().and_then(|l| {
            l.items()
                .map(|i| i.entity_button(id!(button)))
                .find(|ab| ab.clicked(actions))
        });

        if let Some(entity_button) = clicked_entity_button {
            let entity = entity_button.get_entity_id().unwrap();
            self.on_entity_selected(scope, &*entity);
        }

        for action in actions.iter().filter_map(|a| a.as_widget_action()) {
            if action.widget_uid == prompt.widget_uid() {
                match action.cast::<TextInputAction>() {
                    TextInputAction::Change(current) => {
                        self.on_prompt_changed(cx, scope, current);
                    }
                    TextInputAction::Escape => self.on_deselected(),
                    _ => {}
                }
            }

            if action.widget_uid == search_input.widget_uid() {
                match action.cast::<TextInputAction>() {
                    TextInputAction::Change(current) => {
                        self.on_search_changed(cx, scope, current.clone());
                    }
                    TextInputAction::Return(_) => {
                        self.on_search_submit(scope);
                    }
                    TextInputAction::Escape => {
                        self.hide_autocomplete();
                        self.prompt_pending_focus = true;
                    }
                    TextInputAction::KeyFocusLost => {
                        self.hide_autocomplete();
                    }
                    _ => {}
                }
            }

            if let ChatAction::Start(_) = action.cast() {
                self.on_deselected();
            }
        }

        if self.button(id!(deselect_button)).clicked(actions) {
            self.on_deselected();
        }

        for action in actions {
            match action.cast() {
                ModelSelectorAction::ModelSelected(_) | ModelSelectorAction::AgentSelected(_) => {
                    self.on_deselected()
                }
                _ => (),
            }
        }
    }
}

impl PromptInput {
    fn on_prompt_changed(&mut self, cx: &mut Cx, scope: &mut Scope, current: String) {
        if self.was_at_added() && moly_mofa::should_be_visible() {
            self.show_autocomplete(cx, scope);
        } else {
            self.hide_autocomplete();
        }

        self.prev_prompt = current;
    }

    fn on_entity_selected(&mut self, scope: &mut Scope, entity: &ChatEntityId) {
        let mut agent_avatar = self.chat_agent_avatar(id!(agent_avatar));
        let store = scope.data.get_mut::<Store>().unwrap();
        let label = self.label(id!(selected_label));

        match entity {
            ChatEntityId::Agent(agent) => {
                let agent = store.available_agents.get(agent).cloned().unwrap_or_default();
                label.set_text(&agent.name);
                agent_avatar.set_agent(&agent);
            }
            ChatEntityId::ModelFile(file_id) => {
                let store = scope.data.get_mut::<Store>().unwrap();
                let file = store
                    .downloads
                    .get_file(file_id)
                    .expect("selected file not found");
                label.set_text(&file.name);
                agent_avatar.set_visible(false);
            }
        }

        self.entity_selected = Some(entity.clone());
        self.hide_autocomplete();
        self.view(id!(selected_bubble)).set_visible(true);

        let prompt = self.text_input(id!(prompt));
        let prompt_cursor_pos = prompt.borrow().map_or(0, |p| p.get_cursor().head.index);
        if prompt_cursor_pos > 0 {
            let last_char_pos = prompt_cursor_pos - 1;
            let last_char = prompt.text().chars().nth(last_char_pos).unwrap_or_default();

            if last_char == '@' {
                let at_removed = prompt
                    .text()
                    .chars()
                    .enumerate()
                    .filter_map(|(i, c)| if i == last_char_pos { None } else { Some(c) })
                    .collect::<String>();

                prompt.set_text(&at_removed);
                self.prev_prompt = at_removed;
            }
        }

        self.prompt_pending_focus = true;
    }

    fn on_deselected(&mut self) {
        self.entity_selected = None;
        self.view(id!(selected_bubble)).set_visible(false);
    }

    fn on_search_changed(&mut self, cx: &mut Cx, scope: &mut Scope, search: String) {
        // disallow multiline input
        self.text_input(id!(search_input))
            .set_text(&search.replace("\n", " "));

        self.compute_list(cx, scope);
    }

    fn on_search_submit(&mut self, scope: &mut Scope) {
        if let Some(list) = self.list(id!(autocomplete.list)).borrow() {
            let item = list
                .items()
                .nth(self.keyboard_focus_index)
                .expect("item is out of range");

            let button = item.entity_button(id!(button));
            let entity = button.get_entity_id().unwrap();
            self.on_entity_selected(scope, &entity);
        }
    }

    /// Computes the autocomplete list based on the search terms
    fn compute_list(&mut self, cx: &mut Cx, scope: &mut Scope) {
        let search = self.text_input(id!(search_input)).text();
        let mut list = self.list(id!(autocomplete.list));
        let store = scope.data.get_mut::<Store>().unwrap();

        let terms = search
            .split_whitespace()
            .map(|s| s.to_ascii_lowercase())
            .collect::<Vec<_>>();

        let available_agents = store.agents_list();
        let agents: Vec<_> = available_agents
            .iter()
            .map(|agent| ChatEntityRef::Agent(&agent))
            .filter(|entity| {
                terms
                    .iter()
                    .all(|term| entity.name().to_ascii_lowercase().contains(term))
            })
            .collect();

        let agents_len = agents.len();

        let model_files: Vec<_> = store.downloads.downloaded_files
            .iter()
            .map(|f| ChatEntityRef::ModelFile(&f.file))
            .filter(|entity| {
                terms
                    .iter()
                    .all(|term| entity.name().to_ascii_lowercase().contains(term))
            })
            .collect();

        // Build items vector with section labels
        let mut items = Vec::new();
        
        // Add agents section if there are any matching agents
        if !agents.is_empty() {
            let label = WidgetRef::new_from_ptr(cx, self.section_label_template);
            label.set_text_and_redraw(cx, "Agents");
            items.push(label);

            // Add agent items
            items.extend(agents.into_iter().enumerate().map(|(idx, item)| {
                // account for the agents header
                let effective_idx = idx + 1;
                create_entity_button(cx, self.entity_template, item, effective_idx == self.keyboard_focus_index)
            }));
        }

        // Add models section if there are any matching models
        if !model_files.is_empty() {
            let label = WidgetRef::new_from_ptr(cx, self.section_label_template);
            label.set_text_and_redraw(cx, "Models");
            items.push(label);

            // Add model items
            items.extend(model_files.into_iter().enumerate().map(|(idx, item)| {
                // account for the section headers
                let effective_idx = if agents_len > 0 { idx + agents_len + 2 } else { idx + 1 };
                create_entity_button(cx, self.entity_template, item, effective_idx == self.keyboard_focus_index)
            }));
        }

        list.set_items(items);
    }

    fn show_autocomplete(&mut self, cx: &mut Cx, scope: &mut Scope) {
        self.view(id!(autocomplete)).set_visible(true);
        self.search_pending_focus = true;
        self.compute_list(cx, scope);
    }

    fn hide_autocomplete(&mut self) {
        self.view(id!(autocomplete)).set_visible(false);
        self.text_input(id!(search_input)).set_text("");
        self.keyboard_focus_index = 0;
    }

    /// Moves the keyboard focus within the autocomplete list
    fn on_search_keyboard_move(&mut self, cx: &mut Cx, scope: &mut Scope, delta: i32) {
        let list = self.list(id!(autocomplete.list));
        let items_vec: Vec<_> = list.borrow()
            .map(|l| l.items().cloned().collect())
            .expect("The autocomplete list was not found");

        if list.len() == 0 {
            return;
        }

        // Move the focus within the list skipping over section headers
        let mut new_index = self.keyboard_focus_index;
        new_index = new_index.saturating_add_signed(delta as isize).clamp(0, list.len() - 1);
        if let Some(item) = items_vec.get(new_index) {
            // The widget is a label (section header), move into the next item
            if item.as_label().borrow().is_some() {
                new_index = new_index.saturating_add_signed(delta as isize).clamp(0, list.len() - 1);
            }
        }

        if new_index != self.keyboard_focus_index {
            self.keyboard_focus_index = new_index;
            self.compute_list(cx, scope);
        }
    }

    fn was_at_added(&mut self) -> bool {
        let prompt = self.text_input(id!(prompt));
        let prev = &self.prev_prompt;
        let current = &prompt.text();

        if current.len() != prev.len() + 1 {
            return false;
        }

        // not necessarily the cursor head, but works for this single character use case
        let cursor_pos = prompt.borrow().map_or(0, |p| p.get_cursor().head.index);

        if cursor_pos == 0 {
            return false;
        }

        let inserted_char = current.chars().nth(cursor_pos - 1).unwrap_or_default();

        inserted_char == '@'
    }
}

impl LiveHook for PromptInput {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        let empty_message = if moly_mofa::should_be_visible() {
            "Start typing or tag @model or @agent"
        } else {
            "Start typing"
        };

        self.text_input(id!(prompt)).apply_over(
            cx,
            live! {
                empty_message: (empty_message),
            },
        );
    }
}

impl PromptInputRef {
    pub fn reset_text(&mut self, set_key_focus: bool) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };

        let prompt_input = inner.text_input(id!(prompt));
        prompt_input.set_text("");
        prompt_input.set_cursor(0, 0);

        inner.hide_autocomplete();
        inner.prev_prompt.clear();

        inner.prompt_pending_focus = set_key_focus;
    }
}

fn set_cursor_to_end(text_input: &TextInputRef) {
    let len = text_input.text().chars().count();
    text_input.set_cursor(len, len);
}

fn create_entity_button(
    cx: &mut Cx, 
    template: Option<LivePtr>, 
    item: ChatEntityRef, 
    is_focused: bool
) -> WidgetRef {
    let widget = WidgetRef::new_from_ptr(cx, template);
    let mut button = widget.entity_button(id!(button));
    button.set_entity(item);
    button.set_description_visible(true);

    if is_focused {
        widget.apply_over(
            cx,
            live! {
                draw_bg: {
                    color: #EAECEF88,
                }
            },
        );
    }

    widget
}
