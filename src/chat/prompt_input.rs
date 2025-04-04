use makepad_widgets::*;
use moly_kit::BotId;

use crate::{
    data::{
        capture::CaptureAction, providers::ProviderBot,
        store::Store,
    },
    shared::actions::ChatAction,
};

use super::{
    entity_button::EntityButtonWidgetRefExt, model_selector_item::ModelSelectorAction,
    shared::ChatAgentAvatarWidgetExt,
};

use std::collections::HashMap;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
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
            border_radius: 6.5,
            color: #D0D5DD
        }
        icon_walk: {
            margin: {top: 0, left: -4},
        }
    }

    pub PromptInput = {{PromptInput}} {
        flow: Down,
        height: Fit,
        entity_template: <View> {
            height: Fit,
            button = <EntityButton> { deaf: true, server_url_visible: true }
        }
        section_label_template: <Label> {
            padding: {top: 4., bottom: 4.}
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 10.0},
                color: #98A2B3
            }
        }
        show_bg: true,
        draw_bg: {
            color: #fff
        }

        prompt = <CommandTextInput> {
            popup = {
                padding: {bottom: 12.0, top: 12.0, right: 6.0, left: 6.0},
                margin: {bottom: 10},
                draw_bg: {
                    border_size: 1.0,
                    border_color_1: #D0D5DD,
                    color: #fff,
                    border_radius: 5.0
                }
                search_input = <MolyTextInput> {
                    width: Fill,
                    margin: {bottom: 4},
                    empty_message: "Search for a model or agent",
                    draw_bg: {
                        border_radius: 5.0,
                        color: #fff
                    }
                    draw_text: {
                        text_style: <REGULAR_FONT>{font_size: 10},
                        color: #475467
                    }
                }
            }

            persistent = {
                padding: {top: 8, bottom: 6, left: 4, right: 10}
                draw_bg: {
                    color: #fff,
                    border_radius: 10.0,
                    border_color_1: #D0D5DD,
                    border_size: 1.0,
                }

                top = {
                    selected_bubble = <RoundedView> {
                        visible: false,
                        width: Fit,
                        height: Fit,
                        align: {y: 0.5},
                        margin: {top: 5, bottom: 9, right: 5, left: 5},
                        padding: {left: 10, right: 10, top: 8, bottom: 8}
                        draw_bg: {
                            color: #F2F4F7,
                            border_radius: 10.0,
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
                }

                center = {
                    text_input = <MolyTextInput> {
                        width: Fill,
                        empty_message: "Start typing",
                        draw_bg: {
                            border_radius: 10.0
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

                    right = {
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

            keyboard_focus_color: #EAECEF88,
            pointer_hover_color: #EAECEF44,
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

    #[rust]
    pub selected_bot: Option<BotId>,
}

impl Widget for PromptInput {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }
}

impl WidgetMatchEvent for PromptInput {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let mut prompt = self.command_text_input(id!(prompt));

        if let Some(item) = prompt.item_selected(actions) {
            let entity_button = item.entity_button(id!(button));
            let bot_id = entity_button.get_bot_id();
            if let Some(bot_id) = bot_id {
                self.on_bot_selected(cx, scope, &bot_id);
            }
        }

        if prompt.should_build_items(actions) {
            let store = scope.data.get::<Store>().unwrap();
            let terms = prompt
                .search_text()
                .split_whitespace()
                .map(|s| s.to_ascii_lowercase())
                .collect::<Vec<_>>();

            prompt.clear_items();

            // Get agents and non-agent models
            let agents = store.chats.get_mofa_agents_list(true);
            let non_agent_models = store.chats.get_non_mofa_models_list(true);

            // Group non-agent models by provider URL
            let mut models_by_provider: HashMap<String, Vec<&ProviderBot>> = HashMap::new();
            
            for model in non_agent_models.iter().filter(|m| terms.iter().all(|t| m.name.to_lowercase().contains(t))) {
                models_by_provider
                    .entry(model.provider_url.clone())
                    .or_default()
                    .push(model);
            }

            // Add models grouped by provider
            for (provider_url, models) in models_by_provider {
                if models.is_empty() {
                    continue;
                }

                // Get provider name from the providers map
                let provider_name = store.chats.providers
                    .get(&provider_url)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| "Unknown Provider".to_string());

                // Add provider section label
                let label = WidgetRef::new_from_ptr(cx, self.section_label_template);
                label.set_text(cx, &provider_name);
                prompt.add_unselectable_item(label);

                // Add models for this provider
                for model in models {
                    let option = WidgetRef::new_from_ptr(cx, self.entity_template);
                    let mut entity_button = option.entity_button(id!(button));
                    entity_button.set_bot_id(cx, &model.id);
                    entity_button.set_description_visible(cx, true);
                    prompt.add_item(option);
                }
            }
            
            // Add agents
            for (idx, agent) in agents
                .iter()
                .filter(|a| terms.iter().all(|t| a.name.to_lowercase().contains(t)))
                .enumerate()
            {
                if idx == 0 {
                    let label = WidgetRef::new_from_ptr(cx, self.section_label_template);
                    label.set_text(cx, "Agents");
                    prompt.add_unselectable_item(label);
                }

                let option = WidgetRef::new_from_ptr(cx, self.entity_template);
                let mut entity_button = option.entity_button(id!(button));
                entity_button.set_bot_id(cx, &agent.id);
                entity_button.set_description_visible(cx, true);
                prompt.add_item(option);
            }

            // Add local models
            for (idx, file) in store
                .downloads
                .downloaded_files
                .iter()
                .map(|f| &f.file)
                .filter(|f| terms.iter().all(|t| f.name.to_lowercase().contains(t)))
                .enumerate()
            {
                if idx == 0 {
                    let label = WidgetRef::new_from_ptr(cx, self.section_label_template);
                    label.set_text(cx, "Local models");
                    prompt.add_unselectable_item(label);
                }

                let option = WidgetRef::new_from_ptr(cx, self.entity_template);
                let mut entity_button = option.entity_button(id!(button));
                let bot_id = store.chats.available_bots.iter().find(|(_, m)| m.name == file.id).map(|(id, _)| id);
                if let Some(bot_id) = bot_id {
                    entity_button.set_bot_id(cx, &bot_id);
                    entity_button.set_description_visible(cx, true);
                    prompt.add_item(option);
                }
            }
        }

        if prompt.text_input_ref().escape(actions) {
            self.on_deselected(cx);
        }

        if self.button(id!(deselect_button)).clicked(actions) {
            self.on_deselected(cx);
        }

        for action in actions {
            match action.cast() {
                ModelSelectorAction::BotSelected(_) => {
                    self.on_deselected(cx);
                }
                _ => (),
            }

            match action.cast() {
                CaptureAction::Capture { event } => {
                    self.command_text_input(id!(prompt))
                        .set_text(cx, event.contents());
                }
                _ => {}
            }
        }

        for action in actions.iter().filter_map(|a| a.as_widget_action()) {
            if let ChatAction::Start(_) = action.cast() {
                self.on_deselected(cx);
            }
        }
    }
}

impl PromptInput {
    fn on_bot_selected(&mut self, cx: &mut Cx, scope: &mut Scope, bot_id: &BotId) {
        let store = scope.data.get::<Store>().unwrap();

        let mut agent_avatar = self.chat_agent_avatar(id!(agent_avatar));
        let label = self.label(id!(selected_label));

        let bot = store.chats.get_bot_or_placeholder(bot_id);
        label.set_text(cx, &bot.name);
        agent_avatar.set_visible(false);

        self.selected_bot = Some(bot_id.clone());
        self.view(id!(selected_bubble)).set_visible(cx, true);
    }

    fn on_deselected(&mut self, cx: &mut Cx) {
        self.selected_bot = None;
        self.view(id!(selected_bubble)).set_visible(cx, false);
    }
}

impl LiveHook for PromptInput {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        let prompt = self.command_text_input(id!(prompt));
        prompt.apply_over(cx, live! { trigger: "@" });
        prompt.text_input_ref().apply_over(
        cx,
            live! {
                empty_message: "Start typing or tag @model or @agent"
            },
        );
    }
}

impl PromptInputRef {
    pub fn reset_text(&mut self, cx: &mut Cx, set_key_focus: bool) {
        let prompt = self.command_text_input(id!(prompt));

        if set_key_focus {
            prompt.request_text_input_focus();
        }

        prompt.reset(cx);
    }
}
