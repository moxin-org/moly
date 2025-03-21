use crate::{
    data::{chats::chat_entity::ChatEntityId, store::Store},
    shared::{
        actions::ChatAction,
        utils::{format_model_size, hex_rgb_color},
    },
};
use makepad_widgets::*;

use super::{
    model_selector_item::ModelSelectorAction, model_selector_list::ModelSelectorListWidgetExt,
    model_selector_loading::ModelSelectorLoadingWidgetExt, shared::ChatAgentAvatarWidgetRefExt,
};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::chat::model_info::ModelInfo;
    use crate::chat::model_info::AgentInfo;
    use crate::chat::model_selector_list::ModelSelectorList;
    use crate::chat::model_selector_loading::ModelSelectorLoading;

    ICON_DROP = dep("crate://self/resources/images/drop_icon.png")


    ModelSelectorButton = <RoundedView> {
        width: Fill,
        height: 54,
        flow: Overlay,

        loading = <ModelSelectorLoading> {
            width: Fill,
            height: Fill,
            visible: false,
        }

        draw_bg: {
            instance radius: 3.0,
            color: #F9FAFB,
        }

        <View> {
            width: Fill,
            height: Fill,
            flow: Right,

            align: {x: 0.0, y: 0.5},
            padding: {left: 16, right: 16, top: 0, bottom: 0},

            cursor: Hand,

            content = <View> {
                width: Fill,
                height: Fit,
                flow: Overlay,
                padding: {left: 16, top: 0, bottom: 0, right: 0},

                choose = <View> {
                    width: Fill,
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

                selected_model = <ModelInfo> {
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

                selected_agent = <AgentInfo> {
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
                margin: {left: 10, right: 6},

                icon = <RotatedImage> {
                    height: 14,
                    width: 14,
                    source: (ICON_DROP),
                    draw_bg: {
                        rotation: 0.0
                    }
                }
            }
        }
    }

    ModelSelectorOptions = <RoundedView> {
        width: Fill,
        height: 0,

        margin: { top: 5 },
        padding: 5,

        draw_bg: {
            instance radius: 3.0,
            color: #fff,
            border_color: #D0D5DD,
            border_width: 1.0,
        }

        list_container = <View> {
            width: Fill,
            height: 0,
            scroll_bars: <ScrollBars> {}

            list = <ModelSelectorList> {
                width: Fill,
                height: Fit,
            }
        }
    }

    pub ModelSelector = {{ModelSelector}} {
        width: Fill,
        height: Fit,

        flow: Down,

        button = <ModelSelectorButton> {}
        options = <ModelSelectorOptions> {}

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
    currently_selected_model: Option<ChatEntityId>,
}

impl Widget for ModelSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        if let Hit::FingerDown(fd) =
            event.hits_with_capture_overload(cx, self.view(id!(button)).area(), true)
        {
            if fd.tap_count == 1 {
                self.open = !self.open;

                if self.open {
                    let list = self.model_selector_list(id!(options.list_container.list));
                    let height = list.get_height();
                    if height > MAX_OPTIONS_HEIGHT {
                        self.options_list_height = Some(MAX_OPTIONS_HEIGHT);
                    } else {
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
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();
        let choose_label = self.label(id!(choose.label));

        self.update_loading_model_state(cx, store);

        if self.currently_selected_model.is_none() {
            self.view(id!(choose)).set_visible(cx, true);
            self.view(id!(selected_agent)).set_visible(cx, false);
            self.view(id!(selected_model)).set_visible(cx, false);
            choose_label.set_text(cx, "Choose a Model or Agent");
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
        if let Some(fd) = self.view(id!(button)).finger_down(&actions) {
            if fd.tap_count == 1 {
                self.open = !self.open;

                if self.open {
                    let list = self.model_selector_list(id!(options.list_container.list));
                    let height = list.get_height();
                    if height > MAX_OPTIONS_HEIGHT {
                        self.options_list_height = Some(MAX_OPTIONS_HEIGHT);
                    } else {
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

        let mut should_hide_options = false;
        for action in actions {
            match action.cast() {
                ModelSelectorAction::AgentSelected(agent) => {
                    self.currently_selected_model = Some(ChatEntityId::Agent(agent.id));
                    should_hide_options = true;
                }
                ModelSelectorAction::ModelSelected(m) => {
                    self.currently_selected_model = Some(ChatEntityId::ModelFile(m.file.id));
                    should_hide_options = true;
                }
                ModelSelectorAction::RemoteModelSelected(m) => {
                    self.currently_selected_model = Some(ChatEntityId::RemoteModel(m.id));
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
        self.redraw(cx);
    }

    fn update_loading_model_state(&mut self, cx: &mut Cx, store: &Store) {
        let is_associated_model_loading = store
            .chats
            .get_current_chat()
            .and_then(|c| match &c.borrow().associated_entity {
                Some(ChatEntityId::ModelFile(file_id)) => Some(file_id.clone()),
                _ => None,
            })
            .map(|file_id| {
                let model_loader = &store.chats.model_loader;
                model_loader.file_id() == Some(file_id) && model_loader.is_loading()
            })
            .unwrap_or(false);

        if is_associated_model_loading {
            self.model_selector_loading(id!(loading))
                .show_and_animate(cx);
        } else {
            self.model_selector_loading(id!(loading)).hide();
        }
    }

    fn update_selected_model_info(&mut self, cx: &mut Cx, store: &Store) {
        self.view(id!(choose)).set_visible(cx, false);

        let is_loading = store.chats.model_loader.is_loading();
        let loaded_file = store.chats.loaded_model.as_ref();

        // let chat_entity = self.currently_selected_model.clone();
        let chat_entity = store.chats.get_current_chat().and_then(|c| c.borrow().associated_entity.clone());
        
        // Handle agent
        if let Some(ChatEntityId::Agent(model_id)) = chat_entity {
            self.view(id!(selected_model)).set_visible(cx, false);
            let selected_view = self.view(id!(selected_agent));
            selected_view.set_visible(cx, true);

            let agent = store.chats.get_remote_model_or_placeholder(&model_id);
            selected_view.apply_over(
                cx,
                live! {
                    label = { text: (&agent.name) }
                },
            );
            selected_view
                .chat_agent_avatar(id!(avatar))
                .set_agent(&agent);

            return;
        }

        // Handle remote model
        if let Some(ChatEntityId::RemoteModel(model_id)) = chat_entity {
            self.view(id!(selected_agent)).set_visible(cx, false);
            let selected_view = self.view(id!(selected_model));
            selected_view.set_visible(cx, true);
    
            let remote_model = store.chats.get_remote_model_or_placeholder(&model_id);
            
            selected_view.apply_over(
                cx,
                live! {
                    label = { text: (&remote_model.name.trim_start_matches("models/")), draw_text: { color: #x0 }}
                    // Hide size/architecture tags for remote models
                    architecture_tag = { visible: false }
                    params_size_tag = { visible: false }
                    file_size_tag = { visible: false }
                },
            );
            return;
        }

        // Handle local model
        let file = match chat_entity {
            Some(ChatEntityId::ModelFile(file_id)) => store.downloads.get_file(&file_id).cloned(),
            _ => loaded_file.cloned(),
        };

        let is_file_in_loader =
            store.chats.model_loader.file_id().as_ref() == file.as_ref().map(|f| &f.id);

        if let Some(file) = file {
            self.view(id!(selected_agent)).set_visible(cx, false);
            let selected_view = self.view(id!(selected_model));
            selected_view.set_visible(cx, true);

            let text_color = if Some(&file.id) == loaded_file.map(|f| &f.id) {
                hex_rgb_color(0x000000)
            } else {
                hex_rgb_color(0x667085)
            };

            let caption = if is_loading && is_file_in_loader {
                format!("Loading {}", file.name.trim())
            } else {
                file.name.trim().to_string()
            };

            let file_size = format_model_size(file.size.trim()).unwrap_or("".into());
            let is_file_size_visible = !file_size.is_empty() && !is_loading;

            selected_view.apply_over(
                cx,
                live! {
                    label = { text: (caption), draw_text: { color: (text_color) }}
                    file_size_tag = { visible: (is_file_size_visible), caption = { text: (file_size), draw_text: { color: (text_color) }}}
                },
            );

            if let Some(model) = store.downloads.get_model_by_file_id(&file.id) {
                let architecture = model.architecture.trim();
                let params_size = model.size.trim();
                let is_architecture_visible = !architecture.is_empty() && !is_loading;
                let is_params_size_visible = !params_size.is_empty() && !is_loading;

                selected_view.apply_over(
                    cx,
                    live! {
                        architecture_tag = { visible: (is_architecture_visible), caption = { text: (architecture), draw_text: { color: (text_color) }}}
                        params_size_tag = { visible: (is_params_size_visible), caption = { text: (params_size), draw_text: { color: (text_color) }}}
                    },
                );
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
    pub fn set_currently_selected_model(&mut self, model: Option<ChatEntityId>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.currently_selected_model = model;
            // inner.should_update_selected_model_info = true;
        }
    }
}
