use makepad_widgets::*;
use moxin_mae::MaeAgent;

use crate::{
    chat::shared::ChatAgentAvatarWidgetRefExt,
    shared::{list::ListWidgetExt, meta::MetaWidgetRefExt},
};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::list::*;
    import crate::shared::meta::*;
    import crate::shared::widgets::*;
    import crate::shared::styles::*;

    import crate::chat::shared::ChatAgentAvatar;

    COLLAPSED_HEIGHT = 45;
    EXPANDED_HEIGHT = (COLLAPSED_HEIGHT * 3);

    AgentSelector = {{AgentSelector}} {
        height: Fit,
        agent_template: <View> {
            flow: Overlay,
            height: 45,
            agent = <Meta> {}
            <View> {
                align: {x: 0.5, y: 0.5},
                spacing: 10,
                avatar = <ChatAgentAvatar> {}
                text = <Label> {
                    draw_text: {
                        text_style: <BOLD_FONT> { font_size: 10 },
                        color: #000,
                    }
                }
            }
            button = <MoxinButton> {
                width: Fill,
                height: Fill,
                draw_bg: {
                    radius: 0.0,
                    border_width: 0.0,
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

#[derive(Live, Widget, LiveHook)]
pub struct AgentSelector {
    #[deref]
    view: View,

    #[live]
    agent_template: Option<LivePtr>,

    #[animator]
    animator: Animator,

    #[rust]
    agents: Vec<MaeAgent>,

    #[rust]
    recompute: bool,
}

impl Widget for AgentSelector {
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

impl WidgetMatchEvent for AgentSelector {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let clicked_agent = self
            .list(id!(list))
            .borrow()
            .map(|list| {
                list.items()
                    .find(|item| item.button(id!(button)).clicked(actions))
                    .map(|item| *item.meta(id!(agent)).get_value::<MaeAgent>().unwrap())
            })
            .flatten();

        if let Some(agent) = clicked_agent {
            self.set_agent(agent);
            self.recompute_list(cx);
            self.toggle_layout_mode(cx);
            self.redraw(cx);
        }
    }
}

impl AgentSelector {
    fn toggle_layout_mode(&mut self, cx: &mut Cx) {
        if self.animator.animator_in_state(cx, id!(mode.collapsed)) {
            self.animator_play(cx, id!(mode.expanded));
        } else {
            self.animator_play(cx, id!(mode.collapsed));
        }
    }

    pub fn selected_agent(&self) -> Option<MaeAgent> {
        self.list(id!(list))
            .borrow()
            .map(|list| {
                list.items()
                    .next()
                    .map(|item| *item.meta(id!(agent)).get_value::<MaeAgent>().unwrap())
            })
            .flatten()
    }

    pub fn set_agents(&mut self, agents: Vec<MaeAgent>) {
        self.agents = agents;
        self.recompute = true;
    }

    pub fn set_agent(&mut self, agent: MaeAgent) {
        self.agents = std::iter::once(agent)
            .chain(self.agents.iter().copied().filter(|a| *a != agent))
            .collect();

        self.recompute = true;
    }

    fn recompute_list(&self, cx: &mut Cx) {
        let items = self.agents.iter().copied().map(|a| {
            let widget = WidgetRef::new_from_ptr(cx, self.agent_template);
            widget.label(id!(text)).set_text(&a.name());
            widget.chat_agent_avatar(id!(avatar)).set_agent(&a);
            widget.meta(id!(agent)).set_value(a);
            widget
        });

        self.list(id!(list)).set_items(items.collect());
    }
}

impl AgentSelectorRef {
    pub fn set_agents(&self, agents: Vec<MaeAgent>) {
        self.borrow_mut().map(|mut inner| inner.set_agents(agents));
    }

    pub fn set_agent(&self, agent: MaeAgent) {
        self.borrow_mut().map(|mut inner| inner.set_agent(agent));
    }

    pub fn selected_agent(&self) -> Option<MaeAgent> {
        self.borrow().map(|inner| inner.selected_agent()).flatten()
    }
}
