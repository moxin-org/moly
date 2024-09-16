use makepad_widgets::*;
use moxin_mae::MaeAgent;

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

#[derive(Live, LiveHook, Widget)]
pub struct AgentSelector {
    #[deref]
    view: View,

    #[rust]
    layout_mode: LayoutMode,

    #[rust]
    pending_agents_update: Option<Vec<MaeAgent>>,

    #[live]
    agent_template: Option<LivePtr>,
}

impl Widget for AgentSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if let Some(agents) = self.pending_agents_update.take() {
            self.computed_list(id!(list))
                .compute_from(agents.iter(), |a| {
                    let widget = WidgetRef::new_from_ptr(cx, self.agent_template);
                    widget.button(id!(button)).set_text(&a.name());
                    widget.meta(id!(agent)).set_value(*a);
                    widget
                });
        }

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
        let mut redraw = false;

        if let Some(list) = self.computed_list(id!(list)).borrow() {
            for item in list.items() {
                if item.button(id!(button)).clicked(actions) {
                    let agent = *item.meta(id!(agent)).get_value::<MaeAgent>().unwrap();
                    self.toggle_layout_mode();

                    // Issue: `redraw` will trigger `draw_walk` immediately, where a `WidgetRef` for the list is used,
                    // which hides a `borrow_mut` call internally.
                    // Anyways, this is nice as the runtime borrow checker will prevent us from calling `redraw` in the loop.
                    // self.redraw(cx);
                    redraw = true;
                }
            }
        }

        if redraw {
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
}

impl AgentSelectorRef {
    pub fn set_agents(&mut self, agents: Vec<MaeAgent>) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };

        inner.pending_agents_update = Some(agents);
    }
}
