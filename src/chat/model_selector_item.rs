use makepad_widgets::*;

use crate::data::providers::ProviderBot;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    pub ModelSelectorItem = {{ModelSelectorItem}} {
        width: Fill,
        height: Fit,

        show_bg: true,
        draw_bg: {
            instance hover: 0.0,
            instance down: 0.0,
            color: #fff,
            instance color_hover: #F9FAFB,

            fn pixel(self) -> vec4 {
                return mix(self.color, self.color_hover, self.hover);
            }
        }

        // This is mandatory to listen for touch/click events
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
            down = {
                default: off
                off = {
                    from: {all: Forward {duration: 0.5}}
                    ease: OutExp
                    apply: {
                        draw_bg: {down: 0.0}
                    }
                }
                on = {
                    ease: OutExp
                    from: {
                        all: Forward {duration: 0.2}
                    }
                    apply: {
                        draw_bg: {down: 1.0}
                    }
                }
            }
        }
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ModelSelectorAction {
    BotSelected(ProviderBot),
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelSelectorItem {
    #[deref]
    view: View,

    #[rust]
    model: ProviderBot,
}

impl Widget for ModelSelectorItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ModelSelectorItem {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if let Some(fd) = self.view(id!(content)).finger_down(&actions) {
            if fd.tap_count == 1 {
                cx.action(ModelSelectorAction::BotSelected(self.model.clone()));
            }
        }
    }
}

impl ModelSelectorItemRef {
    pub fn set_bot(&mut self, bot: ProviderBot) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.model = bot;
    }
}
