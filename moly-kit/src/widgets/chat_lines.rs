use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::shaders::*;

    use crate::widgets::message_thinking_block::*;
    use crate::widgets::message_markdown::*;
    use crate::widgets::message_loading::*;
    use crate::widgets::avatar::*;
    use crate::widgets::citation_list::*;


    Sender = <View> {
        height: Fit,
        spacing: 8,
        margin: {bottom: 14},
        align: {y: 0.5}
        avatar = <Avatar> {}
        name = <Label> {
            draw_text:{
                text_style: <THEME_FONT_BOLD>{font_size: 10},
                color: #000
            }
        }
    }


    Bubble = <RoundedView> {
        height: Fit,
        padding: {left: 16, right: 18, top: 18, bottom: 14},
        show_bg: true,
        draw_bg: {
            border_radius: 12.0,
        }
    }

    ActionButton = <Button> {
        width: 14
        height: 14
        icon_walk: {width: 14, height: 14},
        padding: 0,
        draw_icon: {
            color: #BDBDBD,
            color_hover: #x0
            color_down: #ff00
            color_focus: #BDBDBD
        }
        draw_bg: {
            fn pixel(self) -> vec4 {
                return #0000
            }
        }
    }

    Actions = <View> {
        align: {y: 0.5},
        spacing: 6,
        copy = <ActionButton> {
            draw_icon: {
                svg_file: dep("crate://self/resources/copy.svg")
            }
        }
        edit = <ActionButton> {
            draw_icon: {
                svg_file: dep("crate://self/resources/edit.svg")
            }
        }
        delete = <ActionButton> {
            draw_icon: {
                svg_file: dep("crate://self/resources/delete.svg")
            }
        }
    }

    EditActionButton = <Button> {
        padding: {left: 10, right: 10, top: 4, bottom: 4},
        draw_text: {
            color: #000
            color_hover: #000
            color_focus: #000
        }
    }

    EditActions = <View> {
        align: {y: 0.5},
        spacing: 5
        save = <EditActionButton> { text: "save" }
        save_and_regenerate = <EditActionButton> { text: "save and regenerate" }
        cancel = <EditActionButton> { text: "cancel" }
    }

    Editor = <View> {
        height: Fit,
        input = <TextInput> {
            padding: {top: 8, bottom: 8, left: 10, right: 10}
            width: Fill,
            empty_text: "\n",
            draw_bg: {
                color: #fff,
                border_radius: 5.0,
                border_size: 0.0,
                color_focus: #fff
            }

            draw_selection: {
                uniform color: #eee
                uniform color_hover: #ddd
                uniform color_focus: #ddd
            }

            draw_text: {
                color: #x0
                uniform color_hover: #x0
                uniform color_focus: #x0
            }
        }
    }

    pub ChatLine = <RoundedView> {
        flow: Down,
        height: Fit,
        message_section = <RoundedView> {
            flow: Down,
            height: Fit,
            sender = <Sender> {}
            bubble = <Bubble> {}
        }
        actions_section = <View> {
            margin: {top: 4, bottom: 10},
            height: 25,
            actions = <Actions> { visible: false }
            edit_actions = <EditActions> { visible: false }
        }
    }

    pub UserLine = <ChatLine> {
        flow: Down,
        height: Fit,
        message_section = {
            sender = {
                avatar = {
                    grapheme = {
                        draw_bg: {
                            color: #008F7E
                        }
                    }
                }
            }

            bubble = <Bubble> {
                flow: Down,
                padding: 0,
                margin: {left: 32}
                text = <View> {
                    flow: Down
                    height: Fit,
                    label = <Label> {
                        width: Fill,
                        draw_text: {
                            color: #000
                        }
                    }
                }
                editor = <Editor> { visible: false }
            }
        }
        actions_section = {
            margin: {left: 32}
        }
    }

    pub BotLine = <ChatLine> {
        flow: Down,
        height: Fit,
        message_section = {
            bubble = <Bubble> {
                flow: Down,
                padding: 0,
                margin: {left: 32}
                spacing: 10,
                text = <View> {
                    flow: Down
                    height: Fit,
                    spacing: 10
                    thinking_block = <MessageThinkingBlock> {}
                    markdown = <MessageMarkdown> {}
                }
                editor = <Editor> { visible: false }
                citations = <CitationList> { visible: false }
            }
        }
        actions_section = {
            margin: {left: 32}
        }
    }

    pub LoadingLine = <BotLine> {
        message_section = {
            bubble = {
                text = <View> {
                    height: Fit,
                    loading = <MessageLoading> {}
                }
            }
        }
    }

    // Note: For now, let's use bot's apparence for app messages.
    // Idea: With the current design, this can be something centered and fit
    // up to the fill size. If we drop the current design and simplify it, we could
    // just use the bot's design for all messages.
    pub AppLine = <BotLine> {
        message_section = {
            padding: 12,
            draw_bg: {color: #00f3}
            sender = {
                avatar = {
                    grapheme = {draw_bg: {color: #00f3}}
                }
                name = {text: "Application"}
            }
        }
        actions_section = {
            margin: {left: 2}
        }
    }

    pub ErrorLine = <AppLine> {
        message_section = {
            draw_bg: {color: #f003}

            sender = {
                avatar = {
                    grapheme = {draw_bg: {color: #f003}}
                }
            }
        }
    }
}
