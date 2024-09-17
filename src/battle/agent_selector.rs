use makepad_widgets::*;
use moxin_mae::{MaeAgent, MaeBackend};

use crate::shared::{computed_list::ComputedListWidgetExt, meta::MetaWidgetRefExt};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::computed_list::*;
    import crate::shared::meta::*;
    import crate::shared::widgets::*;
    import crate::shared::styles::*;

    COLLAPSED_HEIGHT = 45;
    EXPANDED_HEIGHT = (COLLAPSED_HEIGHT * 3);

    AgentSelector = {{AgentSelector}} {
        height: Fit,
        agent_template: <View> {
            height: Fit,
            agent = <Meta> {}
            button = <MoxinButton> {
                draw_text: {
                    color: #000,
                }
                draw_bg: {
                    radius: 0.0,
                    border_width: 0.0,
                }
                width: Fill,
                height: 45,
            }
        },
        clip = <CachedRoundedView> {
            draw_bg: {
                border_width: 1.25,
                border_color: #D0D5DD,
                radius: 5.0
            },
            <View> {
                show_bg: true,
                draw_bg: {
                    color: #F5F7FA,
                },
                list = <ComputedList> {}
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

#[derive(Live, Widget)]
pub struct AgentSelector {
    #[deref]
    view: View,

    #[live]
    agent_template: Option<LivePtr>,

    #[animator]
    animator: Animator,
}

impl Widget for AgentSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for AgentSelector {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let clicked_agent = self
            .computed_list(id!(list))
            .borrow()
            .map(|list| {
                list.items()
                    .find(|item| item.button(id!(button)).clicked(actions))
                    .map(|item| *item.meta(id!(agent)).get_value::<MaeAgent>().unwrap())
            })
            .flatten();

        if let Some(agent) = clicked_agent {
            self.recompute_list(cx, agent);
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

    fn selected_agent(&self) -> Option<MaeAgent> {
        self.computed_list(id!(list))
            .borrow()
            .map(|list| {
                list.items()
                    .next()
                    .map(|item| *item.meta(id!(agent)).get_value::<MaeAgent>().unwrap())
            })
            .flatten()
    }

    fn recompute_list(&self, cx: &mut Cx, agent: MaeAgent) {
        let agents = MaeBackend::available_agents();
        let agents = agents.iter().filter(|a| **a != agent).copied();
        let agents = std::iter::once(agent).chain(agents);

        self.computed_list(id!(list)).compute_from(agents, |a| {
            let widget = WidgetRef::new_from_ptr(cx, self.agent_template);
            widget.button(id!(button)).set_text(&a.name());
            widget.meta(id!(agent)).set_value(a);
            widget
        });
    }
}

impl LiveHook for AgentSelector {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        if let Some(agent) = MaeBackend::available_agents().first() {
            self.recompute_list(cx, *agent);
        }
    }
}
