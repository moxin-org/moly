use makepad_widgets::*;

use crate::shared::actions::ChatAction;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::chat::chat_history::ChatHistory;

    ICON_NEW_CHAT = dep("crate://self/resources/icons/new_chat.svg")

    HeadingLabel = <Label> {
        margin: {left: 4, bottom: 4},
        draw_text:{
            text_style: <BOLD_FONT>{font_size: 10.5},
            color: #3
        }
    }

    NoAgentsWarning = <Label> {
        margin: {left: 4, bottom: 4},
        width: Fill
        draw_text:{
            text_style: {font_size: 8.5},
            color: #3
        }
    }

    pub ChatHistoryPanel = {{ChatHistoryPanel}} <MolyTogglePanel> {
        // Workaround: Instantiate a view replacing the whole `open_content` content,
        // because `CachedView` is currently rendering up-side-down on web.
        open_content = <View> {
            <ChatHistory> {
                margin: {top: 80}
            }
            right_border = <View> {
                width: 1.6, height: Fill
                margin: {top: 15, bottom: 15}
                show_bg: true,
                draw_bg: {
                    color: #eaeaea
                }
            }
        }

        persistent_content = {
            margin: { left: -10 },
            default = {
                after = {
                    new_chat_button = <MolyButton> {
                        width: Fit,
                        height: Fit,
                        icon_walk: {margin: { top: -1 }, width: 21, height: 21},
                        draw_icon: {
                            svg_file: (ICON_NEW_CHAT),
                            fn get_color(self) -> vec4 {
                                return #475467;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatHistoryPanel {
    #[deref]
    deref: TogglePanel,
}

impl Widget for ChatHistoryPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ChatHistoryPanel {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if self.button(ids!(new_chat_button)).clicked(&actions) {
            cx.action(ChatAction::StartWithoutEntity);
            self.redraw(cx);
        }
    }
}
