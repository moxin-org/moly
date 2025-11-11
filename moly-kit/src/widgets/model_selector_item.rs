use crate::protocol::{Bot, BotId};
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    pub ModelSelectorItem = {{ModelSelectorItem}} {
        width: Fill,
        height: Fit,
        padding: {left: 24, right: 16, top: 12, bottom: 12}
        spacing: 10
        align: {x: 0.0, y: 0.5}

        show_bg: true,
        draw_bg: {
            instance hover: 0.0,
            instance selected: 0.0,
            instance color_hover: #F9FAFB,

            fn pixel(self) -> vec4 {
                return mix(self.color, self.color_hover, self.hover);
            }
        }

        cursor: Hand,

        animator: {
            hover = {
                default: off
                off = {
                    from: {all: Forward {duration: 0.2}}
                    apply: {
                        draw_bg: {hover: 0.0}
                    }
                }

                on = {
                    from: {all: Snap}
                    apply: {
                        draw_bg: {hover: 1.0}
                    },
                }
            }
        }

        label = <Label> {
            width: Fill
            draw_text:{
                text_style: <THEME_FONT_REGULAR>{font_size: 11},
                color: #000
            }
        }

        icon_tick_view = <View> {
            width: Fit, height: Fit
            visible: false
            icon_tick = <Label> {
                width: Fit, height: Fit
                align: {x: 1.0, y: 0.5}
                text: "" // fa-check
                draw_text: {
                    text_style: <THEME_FONT_ICONS> {
                        font_size: 12.
                    }
                    color: #000
                }
            }
        }
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ModelSelectorItemAction {
    BotSelected(BotId),
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelSelectorItem {
    #[deref]
    view: View,

    #[rust]
    bot: Option<Bot>,

    #[rust]
    selected_bot_id: Option<BotId>,

    #[animator]
    animator: Animator,
}

impl Widget for ModelSelectorItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        // Handle tap on the entire item
        match event.hits_with_capture_overload(cx, self.view.area(), true) {
            Hit::FingerDown(_) => {
                self.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerUp(fe) => {
                self.animator_play(cx, ids!(hover.off));
                if fe.was_tap() {
                    if let Some(bot) = &self.bot {
                        cx.action(ModelSelectorItemAction::BotSelected(bot.id.clone()));
                    }
                }
            }
            Hit::FingerHoverIn(_) => {
                self.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                self.animator_play(cx, ids!(hover.off));
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if let Some(bot) = &self.bot {
            self.label(ids!(label)).set_text(cx, &bot.name);

            // Show tick icon if this bot is selected
            let is_selected = self.selected_bot_id.as_ref() == Some(&bot.id);
            self.view(ids!(icon_tick_view)).set_visible(cx, is_selected);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl ModelSelectorItemRef {
    pub fn set_bot(&mut self, bot: Bot) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.bot = Some(bot);
        }
    }

    pub fn set_selected_bot_id(&mut self, selected_bot_id: Option<BotId>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.selected_bot_id = selected_bot_id;
        }
    }
}
