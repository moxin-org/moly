use makepad_widgets::*;
use moly_kit::protocol::*;

use crate::meta::MetaWidgetRefExt;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::meta::*;

    COLLAPSED_HEIGHT = 45;
    EXPANDED_HEIGHT = (COLLAPSED_HEIGHT * 4 + COLLAPSED_HEIGHT / 2);

    pub BotSelector = {{BotSelector}} {
        height: Fit,
        clip = <CachedRoundedView> {
            draw_bg: {
                border_size: 1.0,
                border_color: #D0D5DD,
                border_radius: 5.0
            },
            <View> {
                show_bg: true,
                draw_bg: {
                    color: #F5F7FA,
                },
                list = <PortalList> {
                    height: Fill,
                    width: Fill,
                    Bot = <View> {
                        flow: Overlay,
                        height: 45,
                        bot = <Meta> {}
                        <View> {
                            align: {x: 0.5, y: 0.5},
                            spacing: 10,
                            // avatar = <ChatbotAvatar> {}
                            text = <Label> {
                                draw_text: {
                                    text_style: { font_size: 10 },
                                    color: #000,
                                }
                            }
                        }
                        button = <Button> {
                            width: Fill,
                            height: Fill,
                            draw_bg: {
                                // border_radius: 0.0,
                                // border_size: 0.0,
                            },
                            draw_text: {
                                color: #000,
                            }
                        }
                    },
                }
            }

        },
        animator: {
            mode = {
                default: collapsed,
                collapsed = {
                    redraw: true,
                    from: { all: Forward { duration: 0.20 } }
                    ease: ExpDecay { d1: 0.80, d2: 0.97 }
                    apply: { height: (COLLAPSED_HEIGHT) }
                }
                expanded = {
                    redraw: true,
                    from: { all: Forward { duration: 0.20 } }
                    ease: ExpDecay { d1: 0.80, d2: 0.97 }
                    apply: { height: (EXPANDED_HEIGHT) }
                }
            }
        }
    }
}

#[derive(Debug, Clone, DefaultNone, PartialEq)]
enum InternalAction {
    BotSelected,
    None,
}

#[derive(Live, Widget, LiveHook)]
pub struct BotSelector {
    #[deref]
    view: View,

    #[animator]
    animator: Animator,

    #[rust]
    bots: Vec<Bot>,
}

impl Widget for BotSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(widget) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = widget.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, self.bots.len());

                while let Some(index) = list.next_visible_item(cx) {
                    if index >= self.bots.len() {
                        continue;
                    }

                    let bot = &self.bots[index];

                    let item = list.item(cx, index, live_id!(Bot));
                    item.meta(id!(bot)).set_value(bot.clone());
                    item.button(id!(button)).set_text(cx, &bot.name);
                    item.draw_all(cx, &mut Scope::empty());
                }
            }
        }

        DrawStep::done()
    }
}

impl WidgetMatchEvent for BotSelector {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let clicked_bot_id = self
            .portal_list(id!(list))
            .items_with_actions(actions)
            .iter()
            .find_map(|(_idx, widget)| {
                if widget.button(id!(button)).clicked(actions) {
                    Some(widget.meta(id!(bot)).get_value::<Bot>().unwrap().id.clone())
                } else {
                    None
                }
            });

        if let Some(bot_id) = clicked_bot_id {
            if self.selected_bot_id().as_ref() != Some(&bot_id) {
                self.set_bot(bot_id);
                cx.widget_action(self.widget_uid(), &scope.path, InternalAction::BotSelected);
            }

            self.toggle_layout_mode(cx);
            self.redraw(cx);
        }
    }
}

impl BotSelector {
    fn toggle_layout_mode(&mut self, cx: &mut Cx) {
        if self.animator.animator_in_state(cx, id!(mode.collapsed)) {
            self.animator_play(cx, id!(mode.expanded));
        } else {
            self.animator_play(cx, id!(mode.collapsed));
        }
    }

    pub fn selected_bot_id(&self) -> Option<BotId> {
        self.bots.first().map(|b| b.id.clone())
    }

    pub fn set_bots(&mut self, bots: Vec<Bot>) {
        self.bots = bots;
    }

    pub fn set_bot(&mut self, bot: BotId) {
        let index = self
            .bots
            .iter()
            .position(|b| b.id == bot)
            .expect("bot not found");

        let bot = self.bots.remove(index);
        self.bots.insert(0, bot);

        self.portal_list(id!(list)).set_first_id_and_scroll(0, 0.);
    }

    pub fn bot_selected(&self, actions: &Actions) -> bool {
        actions
            .find_widget_action(self.widget_uid())
            .map(|a| a.cast::<InternalAction>() == InternalAction::BotSelected)
            .unwrap_or(false)
    }
}

impl BotSelectorRef {
    pub fn set_bots(&self, bots: Vec<Bot>) {
        self.borrow_mut().map(|mut inner| inner.set_bots(bots));
    }

    pub fn set_bot(&self, bot: BotId) {
        self.borrow_mut().map(|mut inner| inner.set_bot(bot));
    }

    pub fn selected_bot_id(&self) -> Option<BotId> {
        self.borrow().map(|inner| inner.selected_bot_id()).flatten()
    }

    pub fn bot_selected(&self, actions: &Actions) -> bool {
        self.borrow()
            .map(|inner| inner.bot_selected(actions))
            .unwrap_or(false)
    }
}
