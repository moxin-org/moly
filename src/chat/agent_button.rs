use makepad_widgets::*;
use moly_mofa::MofaAgent;

use super::shared::ChatAgentAvatarWidgetExt;

live_design!(
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::chat::shared::ChatAgentAvatar;

    pub AgentButton = {{AgentButton}} <RoundedView> {
        flow: Right,
        width: Fill,
        visible: false,
        height: 40,
        align: { x: 0.0, y: 0.5 },
        padding: { left: 9, top: 4, bottom: 4, right: 9 },
        spacing: 10,

        cursor: Hand
        show_bg: true,
        draw_bg: {
            radius: 0,
            color: #0000
        }

        agent_avatar = <ChatAgentAvatar> {}
        text_layout = <View> {
            width: Fill,
            height: Fit,
            flow: Right,
            spacing: 10

            caption = <Label> {
                width: Fit,
                height: Fit,
                draw_text: {
                    text_style: <BOLD_FONT>{font_size: 10},
                    color: #000;
                }
            }
            description = <View> {
                visible: false,
                width: Fill,
                height: Fit,
                label = <Label> {
                    width: Fill,
                    height: Fit,
                    draw_text: {
                        wrap: Ellipsis,
                        text_style: <REGULAR_FONT>{font_size: 9, height_factor: 1.1},
                        color: #667085,
                    }
                }
            }
        }
        animator: {
            hover = {
                default: off
                off = {
                    from: {all: Forward {duration: 0.15}}
                    apply: {
                        draw_bg: {color: #F2F4F700}
                    }
                }
                on = {
                    from: {all: Snap}
                    apply: {
                        draw_bg: {color: #EAECEF88}
                    }
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
);

#[derive(Live, Widget, LiveHook)]
pub struct AgentButton {
    #[deref]
    view: View,

    #[rust]
    agent: Option<MofaAgent>,
}

impl Widget for AgentButton {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl AgentButton {
    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.view.widget_uid()) {
            if let ViewAction::FingerDown(fd) = item.cast() {
                return fd.tap_count == 1;
            }
        }

        false
    }

    pub fn get_agent(&self) -> Option<MofaAgent> {
        self.agent
    }

    pub fn set_agent(&mut self, agent: &MofaAgent, show_description: bool) {
        self.visible = true;
        self.label(id!(caption)).set_text(&agent.name());
        self.chat_agent_avatar(id!(agent_avatar)).set_agent(agent);

        self.view(id!(description)).set_visible(show_description);
        if show_description {
            self.view(id!(description.label))
                .set_text(&agent.short_description());
        }

        self.agent = Some(*agent);
    }
}

impl AgentButtonRef {
    pub fn set_agent(&mut self, agent: &MofaAgent, show_description: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_agent(agent, show_description);
        }
    }

    pub fn get_agent(&self) -> Option<MofaAgent> {
        if let Some(inner) = self.borrow() {
            inner.get_agent()
        } else {
            None
        }
    }

    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(inner) = self.borrow() {
            inner.clicked(actions)
        } else {
            false
        }
    }
}
