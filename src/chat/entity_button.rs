use crate::data::{chats::chat_entity::{ChatEntityId, ChatEntityRef}, providers::RemoteModel};

use super::shared::ChatAgentAvatarWidgetExt;
use makepad_widgets::*;

use std::cell::Ref;

live_design!(
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::chat::shared::ChatAgentAvatar;

    pub EntityButton = {{EntityButton}} <RoundedView> {
        flow: Right,
        width: Fill,
        visible: false,
        height: 40,
        align: { x: 0.0, y: 0.5 },
        padding: { left: 9, top: 4, bottom: 4, right: 9 },
        spacing: 10,
        server_url_visible: false,

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
            server_url = <View> {
                visible: false,
                width: Fill,
                height: Fit,
                label = <Label> {
                    draw_text: {
                        text_style: <REGULAR_FONT>{font_size: 9, height_factor: 1.1},
                        color: #667085,
                    }
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
pub struct EntityButton {
    #[deref]
    view: View,

    #[live]
    server_url_visible: bool,

    #[rust]
    entity: Option<ChatEntityId>,

    /// Do not listen to events. Make this fully read-only.
    #[live]
    pub deaf: bool,
}

impl Widget for EntityButton {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.deaf {
            return;
        }

        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.server_url_visible {
            self.view(id!(server_url)).set_visible(cx, true);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl EntityButton {
    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.view.widget_uid()) {
            if let ViewAction::FingerDown(fd) = item.cast() {
                return fd.tap_count == 1;
            }
        }

        false
    }

    #[allow(dead_code)]
    pub fn get_entity_id(&self) -> Option<&ChatEntityId> {
        self.entity.as_ref()
    }

    pub fn set_agent(&mut self, cx: &mut Cx, agent: &RemoteModel) {
        self.set_entity(cx, ChatEntityRef::Agent(agent));
    }

    pub fn set_entity(&mut self, cx: &mut Cx, entity: ChatEntityRef) {
        self.visible = true;


        let name_label = self.label(id!(caption));
        let description_label = self.label(id!(description.label));
        let mut avatar = self.chat_agent_avatar(id!(agent_avatar));
        let server_url = self.label(id!(server_url.label));
        
        // Trim the '/models/' at the beginning of the name (happens with Gemini)
        let name = entity.name().trim_start_matches("models/");
        name_label.set_text(cx, &name);

        if let ChatEntityRef::Agent(agent) = entity {
            avatar.set_visible(true);
            avatar.set_agent(agent);
            description_label.set_text(cx, &agent.description);
            
            let formatted_server_url = agent.provider_url
                .strip_prefix("https://")
                .or_else(|| agent.provider_url.strip_prefix("http://"))
                .unwrap_or(&agent.provider_url);
            server_url.set_text(cx, formatted_server_url);
        } else {
            avatar.set_visible(false);
            description_label.set_text(cx, "");
        }

        self.entity = Some(entity.id());
    }

    pub fn set_description_visible(&mut self, cx: &mut Cx, visible: bool) {
        self.view(id!(description)).set_visible(cx, visible);
    }
}

impl EntityButtonRef {
    pub fn set_description_visible(&mut self, cx: &mut Cx, visible: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_description_visible(cx, visible);
        }
    }

    pub fn set_agent(&mut self, cx: &mut Cx, agent: &RemoteModel) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_agent(cx, agent);
        }
    }

    pub fn set_entity(&mut self, cx: &mut Cx, entity: ChatEntityRef) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_entity(cx, entity);
        }
    }

    pub fn get_entity_id(&self) -> Option<Ref<ChatEntityId>> {
        if let Some(inner) = self.borrow() {
            if inner.entity.is_none() {
                return None;
            }

            Some(Ref::map(inner, |inner| inner.entity.as_ref().unwrap()))
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
