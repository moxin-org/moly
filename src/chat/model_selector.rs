use crate::{
    chat::model_selector_loading::ModelSelectorLoadingWidgetExt,
    data::{
        chats::chat::ChatID,
        store::{ProviderSyncingStatus, Store},
    },
    shared::{actions::ChatAction, modal::ModalWidgetExt, utils::format_model_size},
};
use makepad_widgets::*;
use moly_kit::BotId;

use super::{
    model_selector_item::ModelSelectorAction, model_selector_list::ModelSelectorListWidgetExt,
};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::chat::model_info::ModelInfo;
    use crate::chat::model_info::AgentInfo;
    use crate::shared::modal::*;
    use crate::chat::model_selector_list::ModelSelectorList;
    use crate::chat::model_selector_loading::ModelSelectorLoading;

    ICON_SEARCH = dep("crate://self/resources/icons/search.svg")
    ICON_DROP = dep("crate://self/resources/images/drop_icon.png")

    ModelSelectorButton = <View> {
        width: Fit,
        height: 54,
        flow: Overlay,

        align: {x: 0.0, y: 0.5}
        loading = <ModelSelectorLoading> {}

        <View> {
            width: Fit, height: Fill
            flow: Right, spacing: 8

            align: {x: 0.0, y: 0.5},
            padding: {left: 8, right: 8, top: 0, bottom: 0},

            cursor: Hand,

            content = <View> {
                width: Fit,
                height: Fit,
                flow: Overlay,
                padding: {left: 16, top: 0, bottom: 0, right: 0},

                align: {x: 0.0, y: 0.5},

                choose = <View> {
                    width: Fit,
                    height: Fit,

                    align: {x: 0.0, y: 0.5},
                    padding: 16,

                    label = <Label> {
                        draw_text:{
                            text_style: <BOLD_FONT>{font_size: 11},
                            color: #000
                        }
                        text: "Choose a model"
                    }
                }

                selected_bot = <ModelInfo> {
                    width: Fit,
                    height: Fit,
                    show_bg: false,
                    visible: false,

                    padding: 0,

                    label = {
                        draw_text: {
                            text_style: <BOLD_FONT>{font_size: 11},
                        }
                    }
                }
            }

            icon_drop = <RoundedView> {
                width: Fit,
                height: Fit,
                align: {x: 1.0, y: 0.5},
                margin: {left: 8, right: 8, top: 2},

                icon = <RotatedImage> {
                    height: 12,
                    width: 12,
                    source: (ICON_DROP),
                    draw_bg: {
                        rotation: 0.0
                    }
                }
            }
        }
    }

    ModelSelectorOptions = <RoundedShadowView> {
    flow: Down
        width: Fill, height: Fit,
        padding: 5,

        draw_bg: {
            color: (MAIN_BG_COLOR_DARK),
            border_radius: 4.5,
            uniform shadow_color: #0002
            shadow_radius: 9.0,
            shadow_offset: vec2(0.0,-2.0)
        }

        search = <RoundedView> {
            width: Fill, height: Fit,
            show_bg: true,
            padding: {top: 3, bottom: 3, left: 20, right: 20},
            spacing: 4,
            align: {x: 0.0, y: 0.5},
            draw_bg: {
                border_radius: 6.0,
                border_color: #D0D5DD,
                border_size: 1.0,
                color: #fff,
            }
            <Icon> {
                draw_icon: {
                    svg_file: (ICON_SEARCH),
                    fn get_color(self) -> vec4 { return #666; }
                }
                icon_walk: {width: 14, height: Fit}
            }
            input = <MolyTextInput> {
                width: Fill, height: Fit,
                empty_text: "Search models",
                draw_text: { text_style:<REGULAR_FONT>{font_size: 11} }
            }
        }

        list_container = <View> {
            width: Fill,
            height: 400,
            scroll_bars: <ScrollBars> {}
            padding: 8
            list = <ModelSelectorList> {
                width: Fill,
                height: Fit,
            }
        }
    }

    pub ModelSelector = {{ModelSelector}}<RoundedShadowView> {
        width: Fit, height: Fit
        flow: Down
        margin: {left: 12, right: 12, top: 8, bottom: 15}

        show_bg: true
        draw_bg: {
            color: (MAIN_BG_COLOR_DARK),
            border_radius: 4.5,
            uniform shadow_color: #0001
            shadow_radius: 15.0,
            shadow_offset: vec2(0.0,-2.0)
        }

        button = <ModelSelectorButton> {}
        bot_options_modal = <Modal> {
            align: {x: 0.0, y: 0.0}
            bg_view: {
                visible: false
            }
            content: {
                padding: {top: 20, left: 10, right: 10, bottom: 20}
                width: 368
                height: 500
                options = <ModelSelectorOptions> {}
            }
        }

        open_animation_progress: 0.0,
        rotate_animation_progress: 0.0
        animator: {
            open = {
                default: hide,
                show = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {open_animation_progress: 1.0, rotate_animation_progress: 1.0}
                }
                hide = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {open_animation_progress: 0.0, rotate_animation_progress: 0.0}
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelSelector {
    #[deref]
    view: View,

    #[rust]
    open: bool,

    #[animator]
    animator: Animator,

    #[live]
    open_animation_progress: f64,

    #[live]
    rotate_animation_progress: f64,

    #[rust]
    hide_animation_timer: Timer,

    #[rust]
    options_list_height: Option<f64>,

    #[rust]
    currently_selected_model: Option<BotId>,

    #[rust]
    chat_id: ChatID,
}

impl Widget for ModelSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
        let store = scope.data.get::<Store>().unwrap();

        // Check if we have a model selected, if not but there's an associated bot in the chat, use it
        if self.currently_selected_model.is_none() {
            if let Some(chat) = store.chats.get_chat_by_id(self.chat_id) {
                if let Some(bot_id) = chat.borrow().associated_bot.clone() {
                    // Make sure the bot is still available
                    let bot_available = store
                        .chats
                        .get_all_bots(true)
                        .iter()
                        .any(|bot| &bot.id == &bot_id);

                    if bot_available {
                        self.currently_selected_model = Some(bot_id);
                        self.update_selected_model_info(cx, store);
                        self.redraw(cx);
                    }
                }
            }
        }

        if let Hit::FingerDown(fd) =
            event.hits_with_capture_overload(cx, self.view(ids!(button)).area(), true)
        {
            let is_syncing = matches!(
                store.provider_syncing_status,
                ProviderSyncingStatus::Syncing(_)
            );
            if fd.tap_count == 1 && !store.chats.get_all_bots(true).is_empty() && !is_syncing {
                self.open = !self.open;

                if self.open {
                    let button_rect = self.view(ids!(button)).area().rect(cx);
                    let coords = dvec2(
                        button_rect.pos.x - 5.0,
                        button_rect.pos.y + button_rect.size.y,
                    );
                    let modal_content_size = (button_rect.size.x + 10.0).max(360.0);

                    let modal = self.modal(ids!(bot_options_modal));
                    modal.apply_over(
                        cx,
                        live! {
                            content: { margin: { left: (coords.x), top: (coords.y) }, width: (modal_content_size) }
                        },
                    );
                    modal.open(cx);

                    let list = self.model_selector_list(ids!(list_container.list));
                    let height = list.get_height();
                    if height > MAX_OPTIONS_HEIGHT {
                        self.options_list_height = Some(MAX_OPTIONS_HEIGHT);
                    } else if height != 0.0 {
                        self.options_list_height = Some(height);
                    }

                    self.view(ids!(options)).apply_over(
                        cx,
                        live! {
                            height: Fit,
                        },
                    );

                    self.animator_play(cx, ids!(open.show));
                } else {
                    self.hide_animation_timer = cx.start_timeout(0.3);
                    self.animator_play(cx, ids!(open.hide));
                }
            }
        }

        if self.hide_animation_timer.is_event(event).is_some() {
            // When closing animation is done, hide the wrapper element
            self.view(ids!(options)).apply_over(cx, live! { height: 0 });
            self.redraw(cx);
        }

        if self.animator_handle_event(cx, event).must_redraw() {
            if let Some(total_height) = self.options_list_height {
                let height = self.open_animation_progress * total_height;
                self.view(ids!(options.list_container))
                    .apply_over(cx, live! {height: (height)});

                let rotate_angle = self.rotate_animation_progress * std::f64::consts::PI;
                self.view(ids!(icon_drop.icon))
                    .apply_over(cx, live! {draw_bg: {rotation: (rotate_angle)}});

                self.redraw(cx);
            }
        }

        // Trigger a redraw if the provider syncing status has changed
        if let ProviderSyncingStatus::Syncing(_syncing) = &store.provider_syncing_status {
            self.model_selector_loading(ids!(loading))
                .show_and_animate(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();
        let models = store.chats.get_all_bots(true);

        let syncing_status = store.provider_syncing_status.clone();

        // Check if currently selected model is still available (provider might have been disabled)
        self.check_and_clear_unavailable_model(store);

        // Handle syncing status
        match &syncing_status {
            ProviderSyncingStatus::Syncing(syncing) => {
                self.model_selector_loading(ids!(button.loading))
                    .show_and_animate(cx);
                self.view(ids!(choose)).set_visible(cx, true);
                self.view(ids!(icon_drop)).set_visible(cx, false);
                self.view(ids!(selected_bot)).set_visible(cx, false);
                self.label(ids!(choose.label)).set_text(
                    cx,
                    &format!("Syncing providers... {}/{}", syncing.current, syncing.total),
                );
                let color = vec3(0.0, 0.0, 0.0);
                self.label(ids!(choose.label)).apply_over(
                    cx,
                    live! {
                        draw_text: {
                            color: (color)
                        }
                    },
                );
            }
            ProviderSyncingStatus::NotSyncing | ProviderSyncingStatus::Synced => {
                // Just set the loading component to not visible since there's no hide method
                self.view(ids!(button.loading)).set_visible(cx, false);

                if self.currently_selected_model.is_none() {
                    self.view(ids!(choose)).set_visible(cx, true);
                    self.view(ids!(selected_bot)).set_visible(cx, false);
                    let color = vec3(0.0, 0.0, 0.0);
                    self.label(ids!(choose.label)).apply_over(
                        cx,
                        live! {
                            draw_text: {
                                color: (color)
                            }
                        },
                    );

                    // If there are available bots, prompt the user to choose an assistant
                    if !models.is_empty() {
                        self.label(ids!(choose.label))
                            .set_text(cx, "Choose your AI assistant");
                        self.view(ids!(icon_drop)).set_visible(cx, true);
                    } else {
                        self.label(ids!(choose.label))
                            .set_text(cx, "No assistants available, check your provider settings");
                        self.view(ids!(icon_drop)).set_visible(cx, false);
                    }
                } else {
                    self.update_selected_model_info(cx, store);
                }
            }
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

const MAX_OPTIONS_HEIGHT: f64 = 400.0;

impl WidgetMatchEvent for ModelSelector {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let mut should_hide_options = false;
        for action in actions {
            if let Some(text) = self.text_input(ids!(options.search.input)).changed(actions) {
                self.model_selector_list(ids!(list_container.list))
                    .set_search_filter(cx, &text);
            }

            match action.cast() {
                ModelSelectorAction::BotSelected(chat_id, m) => {
                    if chat_id == self.chat_id {
                        self.currently_selected_model = Some(m.id);
                        should_hide_options = true;
                        self.clear_search(cx);
                    }
                }
                _ => {}
            }

            if should_hide_options {
                self.hide_options(cx);
                self.clear_search(cx);
            }

            match action.cast() {
                ChatAction::Start(_) => {
                    self.hide_options(cx);
                    self.clear_search(cx);
                }
                _ => {}
            }

            let modal = self.modal(ids!(bot_options_modal));
            if modal.dismissed(actions) {
                self.clear_search(cx);
            }
        }
    }
}

impl ModelSelector {
    fn clear_search(&mut self, cx: &mut Cx) {
        self.model_selector_list(ids!(list_container.list))
            .clear_search_filter(cx);
        self.text_input(ids!(options.search.input)).set_text(cx, "");
        self.redraw(cx);
    }

    fn hide_options(&mut self, cx: &mut Cx) {
        self.open = false;
        self.view(ids!(options)).apply_over(cx, live! { height: 0 });
        self.view(ids!(icon_drop.icon))
            .apply_over(cx, live! {draw_bg: {rotation: (0.0)}});
        self.animator_cut(cx, ids!(open.hide));
        let modal = self.modal(ids!(bot_options_modal));
        modal.close(cx);
        self.redraw(cx);
    }

    // Helper method to check if the currently selected model is available
    // and clear it if not. Returns true if the model was cleared.
    fn check_and_clear_unavailable_model(&mut self, store: &Store) -> bool {
        if let Some(bot_id) = &self.currently_selected_model.clone() {
            let bot_available = store
                .chats
                .get_all_bots(true)
                .iter()
                .any(|bot| &bot.id == bot_id);

            if !bot_available {
                self.currently_selected_model = None;
                // TODO: Unsure if we should clear the current chat's associated bot here (set to None)
                return true;
            }
        }
        false
    }

    fn update_selected_model_info(&mut self, cx: &mut Cx, store: &Store) {
        self.view(ids!(choose)).set_visible(cx, false);

        let associated_bot = store
            .chats
            .get_chat_by_id(self.chat_id)
            .and_then(|c| c.borrow().associated_bot.clone());
        if let Some(bot_id) = associated_bot {
            let Some(bot) = store.chats.get_bot(&bot_id) else {
                return;
            };
            self.view(ids!(icon_drop)).set_visible(cx, true);

            // Local model styling
            if store.chats.is_local_model(&bot_id) {
                // TODO: Find a better way to map bot ids into file ids, currently relying
                // on the fact that we use the file id as the name of the bot.
                let file = store.downloads.get_file(&bot.name).cloned();
                if let Some(file) = file {
                    let selected_view = self.view(ids!(selected_bot));
                    selected_view.set_visible(cx, true);

                    let file_size = format_model_size(file.size.trim()).unwrap_or("".into());
                    let is_file_size_visible = !file_size.is_empty();
                    let caption = file.name.trim();

                    selected_view.apply_over(
                        cx,
                        live! {
                            label = { text: (caption) }
                            file_size_tag = { visible: (is_file_size_visible), caption = { text: (file_size) }}
                        },
                    );

                    if let Some(model) = store.downloads.get_model_by_file_id(&file.id) {
                        let architecture = model.architecture.trim();
                        let params_size = model.size.trim();
                        let is_architecture_visible = !architecture.is_empty();
                        let is_params_size_visible = !params_size.is_empty();

                        selected_view.apply_over(
                            cx,
                            live! {
                                architecture_tag = { visible: (is_architecture_visible), caption = { text: (architecture) }}
                                params_size_tag = { visible: (is_params_size_visible), caption = { text: (params_size) }}
                            },
                        );
                    }
                }
            } else {
                // Any other model
                let selected_view = self.view(ids!(selected_bot));
                selected_view.set_visible(cx, true);

                selected_view.apply_over(
                    cx,
                    live! {
                        label = { text: (&bot.human_readable_name()), draw_text: { color: #x0 }}
                        // Hide size/architecture tags for remote models
                        architecture_tag = { visible: false }
                        params_size_tag = { visible: false }
                        file_size_tag = { visible: false }
                    },
                );

                let provider = store
                    .chats
                    .get_bot_provider(self.currently_selected_model.as_ref().unwrap());

                let provider_icon = match provider {
                    Some(provider) => store.get_provider_icon(&provider.name),
                    None => None,
                };

                self.set_provider_icon(cx, provider_icon);

                return;
            }
        }

        self.view(ids!(icon_drop)).apply_over(
            cx,
            live! {
                visible: true
            },
        );
    }

    fn set_provider_icon(&mut self, cx: &mut Cx, provider_icon: Option<LiveDependency>) {
        if let Some(provider_icon) = provider_icon {
            self.view(ids!(selected_bot.provider_image_view))
                .set_visible(cx, true);

            let _ = self
                .image(ids!(selected_bot.provider_image))
                .load_image_dep_by_path(cx, provider_icon.as_str());
        } else {
            self.view(ids!(selected_bot.provider_image_view))
                .set_visible(cx, false);
        }
    }
}

impl ModelSelectorRef {
    pub fn set_currently_selected_model(&mut self, cx: &mut Cx, model: Option<BotId>) {
        if let Some(mut inner) = self.borrow_mut() {
            if model != inner.currently_selected_model {
                inner.currently_selected_model = model;
                inner.redraw(cx);
            }
        }
    }

    pub fn set_chat_id(&mut self, chat_id: ChatID) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.chat_id = chat_id;
            inner
                .model_selector_list(ids!(list_container.list))
                .set_chat_id(chat_id);
        }
    }
}
