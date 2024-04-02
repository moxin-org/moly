use makepad_widgets::*;
use crate::data::store::Store;
use crate::data::chat::Chat;
use crate::chat::model_selector::ModelSelectorAction;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import makepad_draw::shader::std::*;

    import crate::chat::model_selector::ModelSelector;

    ICON_PROMPT = dep("crate://self/resources/icons/prompt.svg")

    ChatLineBody = <View> {
        width: Fill,
        height: Fit,
        spacing: 5,
        flow: Down,

        <View> {
            height: 20,
            align: {x: 0.0, y: 0.5},

            role = <Label> {
                width: Fit,
                height: Fit,
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 10},
                    color: #000
                }
            }
        }

        label = <Label> {
            width: Fill,
            height: Fit,
            padding: {top: 12, bottom: 12},

            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 10},
                color: #000,
                word: Wrap,
            }
            text: "Chat Line"
        }
    }

    UserChatLine = <View> {
        margin: {top: 20, bottom: 5},
        width: Fill,
        height: Fit,

        <View> {
            width: Fit,
            height: Fit,
            margin: {left: 20, right: 20},
    
            <Image> {
                source: dep("crate://self/resources/images/chat_user_icon.png"),
                width: 20,
                height: 20,
            }
        }

        <ChatLineBody> {}
    }

    ModelChatLine = <View> {
        margin: {top: 20, bottom: 5},
        width: Fill,
        height: Fit,

        <View> {
            width: Fit,
            height: Fit,
            margin: {left: 20, right: 20},
    
            <RoundedView> {
                width: 20,
                height: 20,

                show_bg: true,
                draw_bg: {
                    color: #444D9A
                }

                align: {x: 0.5, y: 0.5},

                <Label> {
                    width: Fit,
                    height: Fit,
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 10},
                        color: #fff,
                    }
                    text: "P"
                }
            }
        }

        <ChatLineBody> {}
    }

    ChatPromptInput = <RoundedView> {
        width: Fill,
        height: 50,

        show_bg: true,
        draw_bg: {
            color: #fff
        }

        padding: {top: 3, bottom: 3, left: 4, right: 10}

        spacing: 4,
        align: {x: 0.0, y: 0.5},

        draw_bg: {
            radius: 2.0,
            border_color: #D0D5DD,
            border_width: 1.0,
        }

        prompt = <TextInput> {
            width: Fill,
            height: Fit,

            empty_message: "Search Model by Keyword"
            draw_bg: {
                color: #fff
            }
            draw_text: {
                text_style:<REGULAR_FONT>{font_size: 10},

                instance prompt_enabled: 0.0
                fn get_color(self) -> vec4 {
                    return mix(
                        #D0D5DD,
                        #000,
                        self.prompt_enabled
                    )
                }
            }

            // TODO find a way to override colors
            draw_cursor: {
                instance focus: 0.0
                uniform border_radius: 0.5
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(
                        0.,
                        0.,
                        self.rect_size.x,
                        self.rect_size.y,
                        self.border_radius
                    )
                    sdf.fill(mix(#fff, #bbb, self.focus));
                    return sdf.result
                }
            }

            // TODO find a way to override colors
            draw_select: {
                instance hover: 0.0
                instance focus: 0.0
                uniform border_radius: 2.0
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(
                        0.,
                        0.,
                        self.rect_size.x,
                        self.rect_size.y,
                        self.border_radius
                    )
                    sdf.fill(mix(#eee, #ddd, self.focus)); // Pad color
                    return sdf.result
                }
            }
        }

        prompt_icon = <RoundedView> {
            width: 30,
            height: 30,
            show_bg: true,
            draw_bg: {
                color: #D0D5DD
            }

            padding: {right: 4},
            align: {x: 0.5, y: 0.5},

            <Icon> {
                draw_icon: {
                    svg_file: (ICON_PROMPT),
                    fn get_color(self) -> vec4 {
                        return #fff;
                    }
                }
                icon_walk: {width: 12, height: 12}
            }
        }
    }

    ChatPanel = {{ChatPanel}} {
        width: Fill,
        height: Fill,
        margin: 20,
        spacing: 30,

        flow: Overlay,

        main = <View> {
            visible: false

            width: Fill,
            height: Fill,

            margin: { top: 60 }
            spacing: 30,
            flow: Down,

            chat = <PortalList> {
                width: Fill,
                height: Fill,

                auto_tail: true,

                UserChatLine = <UserChatLine> {}
                ModelChatLine = <ModelChatLine> {}
            }

            <ChatPromptInput> {}
        }

        <ModelSelector> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatPanel {
    #[deref]
    view: View,

    #[rust]
    loaded: bool,

    #[rust]
    auto_scroll_pending: bool,
    #[rust]
    auto_scroll_cancellable: bool,

    #[rust(true)]
    prompt_enabled: bool
}

impl Widget for ChatPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        if let Event::Signal = event {
            let store = scope.data.get_mut::<Store>().unwrap();
            if let Some(chat) = &store.current_chat {
                self.auto_scroll_cancellable = true;
                let list = self.portal_list(id!(chat));

                self.prompt_enabled = !store.current_chat.as_ref().unwrap().is_streaming;
                if self.prompt_enabled {
                    // Scroll to the bottom when streaming is done
                    self.scroll_messages_to_bottom(&list, chat);
                    self.auto_scroll_pending = false;
                    self.enable_prompt_input(cx);
                } else {
                    self.disable_prompt_input(cx);
                }

                if self.auto_scroll_pending {
                    // Scroll to the bottom
                    self.scroll_messages_to_bottom(&list, chat);
                }
            } else {
                //panic!("Unexpected error in the model chat session");
            }

            // Redraw because we expect to see new or updated chat entries
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>().unwrap();
        let chat_history; 
        let model_filename;

        if let Some(chat) = &store.current_chat {
            chat_history = chat.messages.clone();
            model_filename = chat.model_filename.clone();
        } else {
            chat_history = vec![];
            model_filename = "".to_string();
        };
        let chats_count = chat_history.len();

        while let Some(view_item) = self.view.draw_walk(cx, scope, walk).step(){
            if let Some(mut list) = view_item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, chats_count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < chats_count {
                        let chat_line_data = &chat_history[item_id];

                        let item;
                        if chat_line_data.is_assistant() {
                            item = list.item(cx, item_id, live_id!(ModelChatLine)).unwrap();
                            item.label(id!(role)).set_text(&model_filename);
                        } else {
                            item = list.item(cx, item_id, live_id!(UserChatLine)).unwrap();
                            item.label(id!(role)).set_text("You");
                        };

                        item.label(id!(label)).set_text(&chat_line_data.content.trim());
                        item.draw_all(cx, &mut Scope::with_data(&mut chat_line_data.clone()));
                    }
                }
            }
        }

        DrawStep::done()
    }
}

impl WidgetMatchEvent for ChatPanel {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        for action in actions {
            match action.as_widget_action().cast() {
                ModelSelectorAction::Selected(downloaded_file) => {
                    self.loaded = true;
                    self.view(id!(main)).set_visible(true);

                    let store = scope.data.get_mut::<Store>().unwrap();
                    store.load_model(&downloaded_file.file);
                    self.prompt_enabled = true;
                },
                _ => {}
            }
        }

        if let Some(text) = self.text_input(id!(prompt)).changed(actions) {
            if self.prompt_enabled && text.len() > 0 {
                self.enable_prompt_input(cx);
            } else {
                self.disable_prompt_input(cx);
            }
        }

        let list = self.portal_list(id!(chat));
        if self.auto_scroll_cancellable && list.scrolled(actions) {
            // Cancel auto-scrolling if the user scrolls up
            self.auto_scroll_pending = false;
        }

        if self.prompt_enabled {
            if let Some(prompt) = self.text_input(id!(prompt)).returned(actions) {
                if prompt.trim().is_empty() {
                    return;
                }

                self.prompt_enabled = false;
                self.disable_prompt_input(cx);
                let store = scope.data.get_mut::<Store>().unwrap();
                store.send_chat_message(prompt.clone());

                self.text_input(id!(prompt)).set_text_and_redraw(cx, "");

                // Scroll to the bottom when the message is sent
                if let Some(chat) = &store.current_chat {
                    self.scroll_messages_to_bottom(&list, chat);
                }
                self.auto_scroll_pending = true;
                self.auto_scroll_cancellable = false;
            }
        }
    }
}

impl ChatPanel {
    fn enable_prompt_input(&mut self, cx: &mut Cx) {
        let enabled_color = vec3(0.0, 0.0, 0.0);
        self.view(id!(prompt_icon)).apply_over(cx, live!{
            draw_bg: {
                color: (enabled_color)
            }
        });
        self.text_input(id!(prompt)).apply_over(cx, live!{
            draw_text: {
                prompt_enabled: 1.0
            }
        });
    }

    fn disable_prompt_input(&mut self, cx: &mut Cx) {
        let disabled_color = vec3(0.816, 0.835, 0.867); // #D0D5DD
        self.view(id!(prompt_icon)).apply_over(cx, live!{
            draw_bg: {
                color: (disabled_color)
            }
        });
        self.text_input(id!(prompt)).apply_over(cx, live!{
            draw_text: {
                prompt_enabled: 0.0
            }
        });
    }

    fn scroll_messages_to_bottom(&mut self, list: &PortalListRef, chat: &Chat) {
        if chat.messages.is_empty() {
            return;
        }
        list.set_first_id_and_scroll(chat.messages.len() - 1, 0.0);
    }
}