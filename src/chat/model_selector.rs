use crate::{
    chat::model_selector_loading::ModelSelectorLoadingWidgetExt, data::store::{ProviderSyncingStatus, Store}, shared::{
        actions::ChatAction, modal::ModalWidgetExt, utils::format_model_size
    }
};
use makepad_widgets::*;
use moly_kit::BotId;

use super::{
    model_selector_item::ModelSelectorAction, model_selector_list::ModelSelectorListWidgetExt
};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::chat::model_info::ModelInfo;
    use crate::chat::model_info::AgentInfo;
    use crate::shared::modal::*;
    use crate::chat::model_selector_list::ModelSelectorList;
    use crate::chat::model_selector_loading::ModelSelectorLoading;

    ICON_DROP = dep("crate://self/resources/images/drop_icon.png")

    ModelSelectorButton = <View> {
        width: Fill,
        height: 54,
        flow: Overlay,

        align: {x: 0.5, y: 0.5}
        loading = <ModelSelectorLoading> {
            width: Fill,
            height: Fill,
            visible: false,
        }


        <View> {
            width: Fill,
            height: Fill,
            flow: Right,

            align: {x: 0.5, y: 0.5},
            padding: {left: 16, right: 16, top: 0, bottom: 0},

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
        width: Fill, height: Fit,
        padding: 5,

        draw_bg: {
            color: (MAIN_BG_COLOR_DARK),
            border_radius: 4.5,
            uniform shadow_color: #0002
            shadow_radius: 9.0,
            shadow_offset: vec2(0.0,-2.0)
        }

        list_container = <View> {
            width: Fill,
            height: 400,
            scroll_bars: <ScrollBars> {}
            list = <ModelSelectorList> {
                width: Fill,
                height: Fit,
            }
        }
    }

    pub ModelSelector = {{ModelSelector}}<RoundedShadowView> {
        width: 500, height: Fit,
        flow: Down,

        show_bg: true,
        draw_bg: {
            color: (MAIN_BG_COLOR_DARK),
            border_radius: 4.5,
            uniform shadow_color: #0001
            shadow_radius: 8.0,
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
                width: 510
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
}

impl Widget for ModelSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
        let store = scope.data.get::<Store>().unwrap();

        if let Hit::FingerDown(fd) =
            event.hits_with_capture_overload(cx, self.view(id!(button)).area(), true)
        {
            let is_syncing = matches!(store.provider_syncing_status, ProviderSyncingStatus::Syncing(_));
            if fd.tap_count == 1 && !store.chats.available_bots.is_empty() && !is_syncing {
                self.open = !self.open;

                if self.open {
                    let button_rect = self.view(id!(button)).area().rect(cx);
                    let coords = dvec2(
                        button_rect.pos.x - 5.0,
                        button_rect.pos.y + button_rect.size.y,
                    );

                    let modal = self.modal(id!(bot_options_modal));
                    modal.apply_over(
                        cx,
                        live! {
                            content: { margin: { left: (coords.x), top: (coords.y) } }
                        },
                    );
                    modal.open(cx);

                    let list = self.model_selector_list(id!(list_container.list));
                    let height = list.get_height();
                    if height > MAX_OPTIONS_HEIGHT {
                        self.options_list_height = Some(MAX_OPTIONS_HEIGHT);
                    } else if height != 0.0  {
                        self.options_list_height = Some(height);
                    }

                    self.view(id!(options)).apply_over(
                        cx,
                        live! {
                            height: Fit,
                        },
                    );

                    self.animator_play(cx, id!(open.show));
                } else {
                    self.hide_animation_timer = cx.start_timeout(0.3);
                    self.animator_play(cx, id!(open.hide));
                }
            }
        }

        if self.hide_animation_timer.is_event(event).is_some() {
            // When closing animation is done, hide the wrapper element
            self.view(id!(options)).apply_over(cx, live! { height: 0 });
            self.redraw(cx);
        }

        if self.animator_handle_event(cx, event).must_redraw() {
            if let Some(total_height) = self.options_list_height {
                let height = self.open_animation_progress * total_height;
                self.view(id!(options.list_container))
                .apply_over(cx, live! {height: (height)});

                let rotate_angle = self.rotate_animation_progress * std::f64::consts::PI;
                self.view(id!(icon_drop.icon))
                    .apply_over(cx, live! {draw_bg: {rotation: (rotate_angle)}});

                self.redraw(cx);
            }
        }

        if let Event::MouseDown(e) = event {
            if self.open {
                let hovered = self.view.area().rect(cx).contains(e.abs);
                if !hovered {
                    self.open = false;
                    self.hide_animation_timer = cx.start_timeout(0.3);
                    self.animator_play(cx, id!(open.hide));
                }
            }
        }

        // Trigger a redraw if the provider syncing status has changed
        if let ProviderSyncingStatus::Syncing(_syncing) = &store.provider_syncing_status {
            // TODO: use the syncing info to show a progress bar instead.
            self.model_selector_loading(id!(loading)).show_and_animate(cx);
        } else {
            self.model_selector_loading(id!(loading)).hide();
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();
        let choose_label = self.label(id!(choose.label));

        if self.currently_selected_model.is_none() {
            self.view(id!(choose)).set_visible(cx, true);
            self.view(id!(selected_bot)).set_visible(cx, false);
            self.view(id!(icon_drop)).set_visible(cx, true);
            choose_label.set_text(cx, "Choose your AI assistant");
            let color = vec3(0.0, 0.0, 0.0);
            choose_label.apply_over(
                cx,
                live! {
                    draw_text: {
                        color: (color)
                    }
                },
            );
        } else if let ProviderSyncingStatus::Syncing(_syncing) = &store.provider_syncing_status {
            self.view(id!(choose)).set_visible(cx, true);
            self.view(id!(icon_drop)).set_visible(cx, false);
            self.view(id!(selected_bot)).set_visible(cx, false);
            choose_label.set_text(cx, "Syncing assistants...");
            let color = vec3(0.0, 0.0, 0.0);
            choose_label.apply_over(
                cx,
                live! {
                    draw_text: {
                        color: (color)
                    }
                },
            );
        } else {
            self.update_selected_model_info(cx, store);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

const MAX_OPTIONS_HEIGHT: f64 = 400.0;

impl WidgetMatchEvent for ModelSelector {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let mut should_hide_options = false;
        for action in actions {
            match action.cast() {
                ModelSelectorAction::BotSelected(m) => {
                    self.currently_selected_model = Some(m.id);
                    should_hide_options = true;
                }
                _ => {}
            }

            if should_hide_options {
                self.hide_options(cx);
            }

            match action.cast() {
                ChatAction::Start(_) => {
                    self.hide_options(cx);
                }
                _ => {}
            }

            let modal = self.modal(id!(bot_options_modal));
            if modal.dismissed(actions) {
                self.hide_options(cx);
            }
        }
    }
}

impl ModelSelector {
    fn hide_options(&mut self, cx: &mut Cx) {
        self.open = false;
        self.view(id!(options)).apply_over(cx, live! { height: 0 });
        self.view(id!(icon_drop.icon))
            .apply_over(cx, live! {draw_bg: {rotation: (0.0)}});
        self.animator_cut(cx, id!(open.hide));
        let modal = self.modal(id!(bot_options_modal));
        modal.close(cx);
        self.redraw(cx);
    }

    fn update_selected_model_info(&mut self, cx: &mut Cx, store: &Store) {
        self.view(id!(choose)).set_visible(cx, false);

        let associated_bot = store.chats.get_current_chat().and_then(|c| c.borrow().associated_bot.clone());
        if let Some(bot_id) = associated_bot {

            let Some(bot) = store.chats.get_bot(&bot_id) else { 
                return;
            };
            self.view(id!(icon_drop)).set_visible(cx, true);

            // Local model styling
            if store.chats.is_local_model(&bot_id) {
                // TODO: Find a better way to map bot ids into file ids, currently relying
                // on the fact that we use the file id as the name of the bot.
                let file = store.downloads.get_file(&bot.name).cloned();
                if let Some(file) = file {
                    let selected_view = self.view(id!(selected_bot));
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
                let selected_view = self.view(id!(selected_bot));
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
                return;
            }
        }        

        self.view(id!(icon_drop)).apply_over(
            cx,
            live! {
                visible: true
            },
        );
    }
}

impl ModelSelectorRef {
    pub fn set_currently_selected_model(&mut self, cx: &mut Cx, model: Option<BotId>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.currently_selected_model = model;
            inner.redraw(cx);
        }
    }
}
