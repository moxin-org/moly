use crate::{avatar::AvatarWidgetRefExt, message_loading::MessageLoadingWidgetRefExt, protocol::*};
use makepad_widgets::*;

// use crate::chat::shared::ChatAgentAvatarWidgetRefExt;

live_design! {
    // import makepad_widgets::base::*;
    // import makepad_widgets::theme_desktop_dark::*;
    // import crate::shared::styles::*;
    // import crate::chat::chat_line_loading::ChatLineLoading;
    // import crate::chat::shared::ChatAgentAvatar;
    // import crate::battle::agent_markdown::AgentMarkdown;

    use link::theme::*;
    use link::widgets::*;

    use crate::message_markdown::*;
    use crate::message_loading::*;
    use crate::avatar::*;


    Bubble = <RoundedView> {
        height: Fit,
        padding: {left: 16, right: 18, top: 18, bottom: 14},
        margin: {bottom: 16},
        show_bg: true,
        draw_bg: {
            radius: 12.0,
        }
    }

    UserLine = <View> {
        height: Fit,
        bubble = <Bubble> {
            margin: {left: 100}
            draw_bg: {color: #15859A}
            text = <Label> {
                width: Fill,
                draw_text: {
                    // text_style: <REGULAR_FONT>{height_factor: (1.3*1.3), font_size: 10},
                    color: #fff
                }
            }
        }
    }

    BotLine = <View> {
        flow: Down,
        height: Fit,
        sender = <View> {
            height: Fit,
            spacing: 8,
            align: {y: 0.5}
            avatar = <Avatar> {}
            name = <Label> {
                draw_text:{
                    // text_style: <BOLD_FONT>{font_size: 10},
                    color: #000
                }
            }
        }
        bubble = <Bubble> {
            margin: {left: 16}
            text = <MessageMarkdown> {}
        }
    }

    LoadingLine = <BotLine> {
        bubble = {
            text = <MessageLoading> {}
        }
    }

    pub Messages = {{Messages}} {
        flow: Down,
        width: Fill,
        height: Fill,

        list = <PortalList> {
            scroll_bar: {
                bar_size: 0.0,
            }
            UserLine = <UserLine> {}
            BotLine = <BotLine> {}
            LoadingLine = <LoadingLine> {}
        }
    }
}

pub struct MessagesProps<'a> {
    pub messages: &'a [Message],
    pub bot_client: &'a dyn BotRepo,
}

#[derive(Live, LiveHook, Widget)]
pub struct Messages {
    #[deref]
    view: View,

    #[rust]
    pub messages: Vec<Message>,

    #[rust]
    pub bot_client: Option<Box<dyn BotRepo>>,
}

impl Widget for Messages {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(widget) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = widget.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, self.messages.len());

                while let Some(index) = list.next_visible_item(cx) {
                    if index >= self.messages.len() {
                        continue;
                    }

                    let message = &self.messages[index];

                    match message.from {
                        EntityId::User => {
                            let item = list.item(cx, index, live_id!(UserLine));
                            item.label(id!(text)).set_text(&message.body);
                            item.draw_all(cx, &mut Scope::empty());
                        }
                        EntityId::Bot(id) => {
                            let bot = self
                                .bot_client
                                .as_ref()
                                .expect("no bot client set")
                                .get_bot(id);

                            let name = bot
                                .as_ref()
                                .map(|b| b.name.as_str())
                                .unwrap_or("Unknown bot");
                            let avatar = bot.as_ref().map(|b| b.avatar.clone());

                            let item = if message.is_writing && message.body.is_empty() {
                                let item = list.item(cx, index, live_id!(LoadingLine));

                                item.message_loading(id!(text)).animate(cx);

                                item
                            } else {
                                let item = list.item(cx, index, live_id!(BotLine));
                                // Workaround: Because I had to set `paragraph_spacing` to 0 in `MessageMarkdown`,
                                // we need to add a "blank" line as a workaround.
                                //
                                // Warning: If you ever read the text from this widget and not
                                // from the list, you should remove the unicode character.
                                item.label(id!(text))
                                    .set_text(&message.body.replace("\n\n", "\n\n\u{00A0}\n\n"));

                                item
                            };

                            item.avatar(id!(avatar)).borrow_mut().unwrap().avatar = avatar;
                            item.label(id!(name)).set_text(name);
                            item.draw_all(cx, &mut Scope::empty());

                            // Message::AgentWriting(agent) => {
                            //     let item = list.item(cx, index, live_id!(LoadingLine));
                            //     item.chat_agent_avatar(id!(avatar)).set_agent(agent);
                            //     item.label(id!(name)).set_text(&agent.name());
                            //     item.draw_all(cx, scope);
                            // }
                        }
                    }
                }
            }
        }

        DrawStep::done()
    }
}

impl Messages {
    pub fn scroll_to_bottom(&self, cx: &mut Cx) {
        self.portal_list(id!(list))
            .smooth_scroll_to_end(cx, 10., Some(80));
    }
}

impl MessagesRef {
    pub fn scroll_to_bottom(&self, cx: &mut Cx) {
        if let Some(inner) = self.borrow() {
            inner.scroll_to_bottom(cx);
        }
    }
}
