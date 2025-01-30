use makepad_widgets::*;
use moly_widgets::protocol::*;

use crate::list::ListWidgetExt;
use crate::meta::MetaWidgetRefExt;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::list::*;
    use crate::meta::*;

    COLLAPSED_HEIGHT = 45;
    EXPANDED_HEIGHT = (COLLAPSED_HEIGHT * 3);

    pub BotSelector = {{BotSelector}} {
        height: Fit,
        bot_template: <View> {
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
                    // radius: 0.0,
                    // border_width: 0.0,
                }
            }
        },
        clip = <CachedRoundedView> {
            draw_bg: {
                border_width: 1.0,
                border_color: #D0D5DD,
                radius: 5.0
            },
            <View> {
                show_bg: true,
                draw_bg: {
                    color: #F5F7FA,
                },
                list = <List> {}
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

    #[live]
    bot_template: Option<LivePtr>,

    #[animator]
    animator: Animator,

    #[rust]
    bots: Vec<Bot>,

    #[rust]
    recompute: bool,
}

impl Widget for BotSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        if self.recompute {
            self.recompute_list(cx);
            self.recompute = false;
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for BotSelector {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let clicked_bot = self
            .list(id!(list))
            .borrow()
            .map(|list| {
                list.items()
                    .find(|item| item.button(id!(button)).clicked(actions))
                    .map(|item| item.meta(id!(bot)).get_value::<Bot>().unwrap().clone())
            })
            .flatten();

        if let Some(bot) = clicked_bot {
            self.set_bot(bot.id);
            self.recompute_list(cx);
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
        self.list(id!(list))
            .borrow()
            .map(|list| {
                list.items()
                    .next()
                    .map(|item| item.meta(id!(bot)).get_value::<Bot>().unwrap().id)
            })
            .flatten()
    }

    pub fn set_bots(&mut self, bots: Vec<Bot>) {
        self.bots = bots;
        self.recompute = true;
    }

    pub fn set_bot(&mut self, bot: BotId) {
        let index = self
            .bots
            .iter()
            .position(|b| b.id == bot)
            .expect("bot not found");

        let bot = self.bots.remove(index);
        self.bots.insert(0, bot);

        self.recompute = true;
    }

    pub fn bot_selected(&self, actions: &Actions) -> bool {
        actions
            .find_widget_action(self.widget_uid())
            .map(|a| a.cast::<InternalAction>() == InternalAction::BotSelected)
            .unwrap_or(false)
    }

    fn recompute_list(&self, cx: &mut Cx) {
        let items = self.bots.iter().cloned().map(|b| {
            let widget = WidgetRef::new_from_ptr(cx, self.bot_template);
            widget.label(id!(text)).set_text(&b.name);
            // widget.chat_bot_avatar(id!(avatar)).set_bot(&a);
            widget.meta(id!(bot)).set_value(b);
            widget
        });

        self.list(id!(list)).set_items(items.collect());
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
