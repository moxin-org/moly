use makepad_widgets::*;
use std::sync::{Arc, Mutex};

use crate::{
    Bot, Picture,
    controllers::chat::{ChatController, ChatStateMutation},
    protocol::BotId,
    utils::makepad::events::EventExt,
    widgets::{
        model_selector_item::ModelSelectorItemAction, model_selector_list::ModelSelectorList,
        moly_modal::MolyModalWidgetExt,
    },
};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::widgets::model_selector_list::ModelSelectorList;
    use crate::widgets::moly_modal::MolyModal;

    ICON_DROP = dep("crate://self/resources/drop_icon.png")

    ModelSelectorButton = <Button> {
        width: Fit,
        height: Fit,
        padding: {left: 8, right: 8, top: 6, bottom: 6}
        // reset_hover_on_click: true

        draw_bg: {
            color_down: #0000
            border_radius: 7.
            border_size: 0.
            color_hover: #f2
        }

        draw_text: {
            text_style: <THEME_FONT_REGULAR> {
                font_size: 11.
            }
            color: #222,
            color_hover: #111,
            color_focus: #111
            color_down: #000
        }
    }

    ModelSelectorOptions = <RoundedShadowView> {
        width: Fill, height: Fit,
        padding: 8,
        flow: Down,
        spacing: 8,

        show_bg: true,
        draw_bg: {
            color: #f9,
            border_radius: 6.0,
            uniform shadow_color: #0002
            shadow_radius: 9.0,
            shadow_offset: vec2(0.0,-2.0)
        }

        search_container = <RoundedView> {
            width: Fill, height: Fit,
            show_bg: true,
            padding: {top: 8, bottom: 8, left: 12, right: 12},
            spacing: 8,
            align: {x: 0.0, y: 0.5},
            draw_bg: {
                border_radius: 6.0,
                border_color: #D0D5DD,
                border_size: 1.0,
                color: #fff,
            }

            search_input = <TextInput> {
                width: Fill, height: Fit,
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        return vec4(0.);
                    }
                }
                draw_text: {
                    text_style: <THEME_FONT_REGULAR>{font_size: 11}
                    color: #000
                    color_hover: #98A2B3
                    color_focus: #000
                    color_empty: #98A2B3
                    color_empty_focus: #98A2B3
                    color_empty_hover: #98A2B3
                }
                draw_cursor: {
                    color: #000
                }
                empty_text: "Search models"
            }
        }

        list_container = <ScrollYView> {
            width: Fill,
            height: 500,
            scroll_bars: {
                scroll_bar_y: {
                    drag_scrolling: true,
                    draw_bg: {
                        color: #D9
                        color_hover: #888
                        color_drag: #777
                    }
                }
            }

            list = <ModelSelectorList> {}
        }
    }

    pub ModelSelector = {{ModelSelector}} <View> {
        width: Fit, height: Fit
        flow: Overlay

        button = <ModelSelectorButton> {
            text: "Loading model..."
        }

        modal = <MolyModal> {
            dismiss_on_focus_lost: true
            bg_view: {
                visible: false
            }
            align: {x: 0.0, y: 0.0}

            content: <View> {
                width: 400
                height: Fit
                padding: {top: 20, left: 10, right: 10, bottom: 20}
                options = <ModelSelectorOptions> {}
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelSelector {
    #[deref]
    view: View,

    #[rust]
    pub chat_controller: Option<Arc<Mutex<ChatController>>>,

    #[rust]
    pub open: bool,
}

impl Widget for ModelSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        // Handle button click to open/close modal
        if self.button(ids!(button)).clicked(event.actions()) {
            if !self.open {
                self.open_modal(cx);
            } else {
                self.close_modal(cx);
            }
        }

        // Handle modal dismissal
        if self.moly_modal(ids!(modal)).dismissed(event.actions()) {
            self.close_modal(cx);
            self.clear_search(cx);
            self.button(ids!(button)).reset_hover(cx);
        }

        // On mobile, handle clicks on background view to dismiss modal
        if self.open && !cx.display_context.is_desktop() {
            if let Hit::FingerUp(fe) = event.hits(cx, self.view(ids!(modal.bg_view)).area()) {
                if fe.was_tap() {
                    self.close_modal(cx);
                    self.clear_search(cx);
                    self.button(ids!(button)).reset_hover(cx);
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Read selected bot from controller state (source of truth)
        let selected_bot_id = if let Some(chat_controller) = &self.chat_controller {
            chat_controller.lock().unwrap().state().bot_id.clone()
        } else {
            None
        };

        // Update button text based on selected bot from controller state
        if let Some(bot_id) = &selected_bot_id {
            if let Some(chat_controller) = &self.chat_controller {
                let state = chat_controller.lock().unwrap().state().clone();
                if let Some(bot) = state.bots.iter().find(|b| &b.id == bot_id) {
                    self.button(ids!(button)).set_text(cx, &bot.name);
                } else {
                    // Bot not found in list (e.g., disabled) - show default text
                    self.button(ids!(button))
                        .set_text(cx, "Choose an AI assistant");
                }
            }
        } else {
            // No bot selected, show default text
            self.button(ids!(button))
                .set_text(cx, "Choose an AI assistant");
        }

        // Set the chat controller and selected bot ID on the list before drawing
        if let Some(controller) = &self.chat_controller {
            if let Some(mut list) = self
                .widget(ids!(options.list_container.list))
                .borrow_mut::<ModelSelectorList>()
            {
                list.chat_controller = Some(controller.clone());
                list.selected_bot_id = selected_bot_id;
            }
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ModelSelector {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Handle search input changes
        if let Some(text) = self
            .text_input(ids!(options.search_container.search_input))
            .changed(actions)
        {
            if let Some(mut list) = self
                .widget(ids!(options.list_container.list))
                .borrow_mut::<ModelSelectorList>()
            {
                list.search_filter = text;
                list.items.clear();
                list.total_height = None;
            }
        }

        // Handle bot selection from list items
        for action in actions {
            match action.cast() {
                ModelSelectorItemAction::BotSelected(bot_id) => {
                    // Dispatch mutation to controller
                    if let Some(controller) = &self.chat_controller {
                        controller
                            .lock()
                            .unwrap()
                            .dispatch_mutation(ChatStateMutation::SetBotId(Some(bot_id)));
                    }

                    self.button(ids!(button)).reset_hover(cx);
                    self.close_modal(cx);
                    self.clear_search(cx);
                    self.redraw(cx);
                }
                _ => {}
            }
        }
    }
}

impl ModelSelector {
    fn open_modal(&mut self, cx: &mut Cx) {
        self.open = true;

        // Get button position and size for positioning the modal
        let button_rect = self.button(ids!(button)).area().rect(cx);

        let modal_content_height = 608.0; // list height (500) + search (40) + padding (68)
        let gap = 25.0;

        let modal_x;
        let modal_y;
        let mut bg_view_visible = false;

        // On desktop, align left edge with button, position above with gap
        if cx.display_context.is_desktop() {
            modal_x = button_rect.pos.x - gap; //+ gap; //- button_rect.size.x - gap;
            modal_y = button_rect.pos.y - modal_content_height - gap - 5.0 // gap;
        } else {
            // On mobile, position the modal in the horizontal center, vertical bottom of the screen
            modal_x = 0.0;
            modal_y = cx.display_context.screen_size.y - modal_content_height - 5.0;
            bg_view_visible = true;
        }

        // // Align left edge of modal with left edge of button
        // let modal_x = button_rect.pos.x - button_rect.size.x - gap; // - modal_width;
        // // Position modal above the button
        // let modal_y = button_rect.pos.y - modal_content_height - gap;

        let modal = self.moly_modal(ids!(modal));
        modal.apply_over(
            cx,
            live! {
                bg_view: {
                    visible: (bg_view_visible)
                }
                content: {
                    margin: { left: (modal_x), top: (modal_y) }
                }
            },
        );

        if !cx.display_context.is_desktop() {
            modal.apply_over(
                cx,
                live! {
                    dismiss_on_focus_lost: false
                    content: {
                        width: Fill
                        padding: 0
                    }
                },
            );
        } else {
            modal.apply_over(
                cx,
                live! {
                    content: { width: 400 }
                    padding: {top: 20, left: 10, right: 10, bottom: 20}
                },
            );
        }

        modal.open(cx);
    }

    fn close_modal(&mut self, cx: &mut Cx) {
        self.open = false;
        self.moly_modal(ids!(modal)).close(cx);
    }

    fn clear_search(&mut self, cx: &mut Cx) {
        if let Some(mut list) = self
            .widget(ids!(options.list_container.list))
            .borrow_mut::<ModelSelectorList>()
        {
            list.search_filter.clear();
            list.items.clear();
            list.total_height = None;
        }
        self.text_input(ids!(options.search_container.search_input))
            .set_text(cx, "");
        self.redraw(cx);
    }
}

impl ModelSelectorRef {
    pub fn set_chat_controller(&mut self, controller: Option<Arc<Mutex<ChatController>>>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.chat_controller = controller;
        }
    }

    /// Set a custom grouping function for organizing bots in the list
    ///
    /// By default, bots are grouped by their provider (extracted from BotId).
    /// Applications can provide a custom grouping function to add
    /// provider icons, custom display names, or different grouping logic.
    ///
    /// The grouping function receives a bot and returns a tuple of:
    /// - `group_id`: Unique identifier for the group (used for deduplication and sorting)
    /// - `group_label`: Display name for the group header
    /// - `group_icon`: Optional icon to display next to the group label
    pub fn set_grouping(&mut self, grouping: Option<GroupingFn>) {
        if let Some(inner) = self.borrow_mut() {
            if let Some(mut list) = inner
                .widget(ids!(options.list_container.list))
                .borrow_mut::<ModelSelectorList>()
            {
                list.grouping = grouping;
            }
        }
    }
}

/// Defines how a bot should be grouped in the model selector.
///
/// This struct is returned by the grouping function to specify:
/// - A unique group identifier for deduplication and sorting
/// - A display label shown in the group header
/// - An optional icon displayed next to the group label
#[derive(Clone, Debug)]
pub struct BotGroup {
    /// Unique identifier for the group (used for deduplication and sorting)
    pub id: String,
    /// Display name shown in the group header
    pub label: String,
    /// Optional icon displayed next to the group label
    pub icon: Option<Picture>,
}

/// Callback function that determines how bots are grouped in the model selector.
///
/// Applications can provide a custom grouping function to organize models by provider,
/// capabilities, or any other criteria. The function receives a bot and returns a
/// [`BotGroup`] that specifies how that bot should be grouped.
///
/// # Default Behavior
/// If no grouping function is provided, bots are grouped by their provider
/// (extracted from `BotId.provider()`), using the bot's avatar as the group icon.
///
/// # Example
/// ```ignore
/// use moly_kit::widgets::model_selector::{GroupingFn, BotGroup};
/// use std::sync::Arc;
///
/// // Group by provider with custom names and icons
/// let grouping: GroupingFn = Arc::new(|bot| {
///     let provider_id = get_provider_id(&bot.id);
///     let provider_name = get_friendly_name(&provider_id);
///     let icon = get_provider_icon(&provider_id);
///     BotGroup {
///         id: provider_id,
///         label: provider_name,
///         icon,
///     }
/// });
///
/// model_selector.set_grouping(Some(grouping));
/// ```
pub type GroupingFn = Arc<dyn Fn(&Bot) -> BotGroup + Send + Sync>;

/// Creates a grouping function that queries data on-demand via a lookup closure.
///
/// This is more efficient than capturing data in the grouping closure, as it avoids
/// data duplication and always returns fresh results from the source.
///
/// # Arguments
/// * `lookup` - A closure that takes a BotId and returns grouping information, or None for default
///
/// # Example
/// ```ignore
/// use moly_kit::widgets::model_selector::{create_lookup_grouping, BotGroup};
///
/// let grouping = create_lookup_grouping(|bot_id| {
///     let provider = get_provider_for_bot(bot_id)?;
///     Some(BotGroup {
///         id: provider.id,
///         label: provider.name,
///         icon: provider.icon,
///     })
/// });
/// ```
pub fn create_lookup_grouping<F>(lookup: F) -> GroupingFn
where
    F: Fn(&BotId) -> Option<BotGroup> + Send + Sync + 'static,
{
    Arc::new(move |bot: &Bot| {
        lookup(&bot.id).unwrap_or_else(|| {
            // Default fallback: group by provider from bot ID
            let provider = bot.id.provider();
            BotGroup {
                id: provider.to_string(),
                label: provider.to_string(),
                icon: Some(bot.avatar.clone()),
            }
        })
    })
}
