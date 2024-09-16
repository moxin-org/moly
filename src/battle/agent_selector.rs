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

    AgentSelector = {{AgentSelector}} {
        // height: 60,
        height: Fit,
        show_bg: true,
        draw_bg: {
            color: #7777de,
        },

        agent_template: <View> {
            height: Fit,
            agent = <Meta> {}
            button = <MoxinButton> {
                width: Fill,
                height: 45,
            }
        },

        clip = <View> {
            list = <ComputedList> {}
        }
    }
}

#[derive(Default)]
enum LayoutMode {
    #[default]
    Collapsed,
    Expanded,
}

#[derive(Live, Widget)]
pub struct AgentSelector {
    #[deref]
    view: View,

    #[rust]
    layout_mode: LayoutMode,

    #[live]
    agent_template: Option<LivePtr>,
}

impl Widget for AgentSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        match self.layout_mode {
            LayoutMode::Collapsed => {
                self.view(id!(clip)).apply_over(cx, live! { height: 45 });
            }
            LayoutMode::Expanded => {
                self.view(id!(clip))
                    .apply_over(cx, live! { height: (45 * 3) });
            }
        }

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
            self.toggle_layout_mode();
            self.redraw(cx);
        }
    }
}

impl AgentSelector {
    fn toggle_layout_mode(&mut self) {
        self.layout_mode = match self.layout_mode {
            LayoutMode::Collapsed => LayoutMode::Expanded,
            LayoutMode::Expanded => LayoutMode::Collapsed,
        };
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
