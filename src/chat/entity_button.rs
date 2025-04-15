use crate::data::store::Store;

use super::shared::ChatAgentAvatarWidgetExt;
use makepad_widgets::*;
use moly_kit::BotId;

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
            border_radius: 0,
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
                        text_style: <REGULAR_FONT>{font_size: 9},
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
                        text_style: <REGULAR_FONT>{font_size: 9},
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
    bot_id: Option<BotId>,

    #[rust]
    should_update_bot_info: bool,

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

        if self.should_update_bot_info && self.bot_id.is_some() {
            self.update_bot_info(cx, scope);
            self.should_update_bot_info = false;
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

    pub fn update_bot_info(&mut self, cx: &mut Cx, scope: &mut Scope) {
        let bot_id = match &self.bot_id {
            Some(bot_id) => bot_id.clone(),
            None => return,
        };

        let store = scope.data.get_mut::<Store>().unwrap();
        self.visible = true;

        let description_label = self.label(id!(description.label));
        let name_label = self.label(id!(caption));
        let mut avatar = self.chat_agent_avatar(id!(agent_avatar));
        let server_url = self.label(id!(server_url.label));


        let bot = store.chats.get_bot_or_placeholder(&bot_id);

        let name = bot.human_readable_name();
        name_label.set_text(cx, &name);

        if store.chats.is_agent(&bot_id) {
            avatar.set_visible(true);
            avatar.set_bot(bot);
            description_label.set_text(cx, &bot.description);

            let formatted_server_url = bot.provider_url
                .strip_prefix("https://")
                .or_else(|| bot.provider_url.strip_prefix("http://"))
                .unwrap_or(&bot.provider_url);
            server_url.set_text(cx, formatted_server_url);
        } else {
            avatar.set_visible(false);
            description_label.set_text(cx, "");
        }
    }

    pub fn set_bot_id(&mut self, cx: &mut Cx, bot_id: &BotId) {
        self.bot_id = Some(bot_id.clone());
        self.should_update_bot_info = true;
        self.redraw(cx);
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

    pub fn set_bot_id(&mut self, cx: &mut Cx, bot_id: &BotId) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_bot_id(cx, bot_id);
        }
    }

    pub fn get_bot_id(&self) -> Option<BotId> {
        if let Some(inner) = self.borrow() {
            inner.bot_id.clone()
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
