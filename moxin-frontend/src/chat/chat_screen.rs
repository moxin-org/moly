use makepad_widgets::*;
use crate::data::store::Store;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import makepad_draw::shader::std::*;

    ChatScreen = {{ChatScreen}} {
        width: Fill,
        height: Fill,
        margin: 50,
        spacing: 30,

        flow: Down,

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 20},
                color: #000
            }
            text: "Chat"
        }

        load_button = <Button> {
            width: 200,
            height: 30,
            text: "Load Model"
        }

        main = <View> {
            visible: false

            width: Fill,
            height: Fill,

            spacing: 30,
            flow: Down,

            chat = <PortalList> {
                width: Fill,
                height: Fill,

                ChatLine = <View> {
                    margin: {bottom: 5},
                    padding: 20,
                    width: Fill,
                    height: Fit,

                    show_bg: true,
                    draw_bg: {
                        color: #ddd
                    }

                    label = <Label> {
                        width: Fill,
                        draw_text:{
                            text_style: <REGULAR_FONT>{font_size: 12},
                            color: #000,
                            word: Wrap,
                        }
                        text: "Chat Line"
                    }
                }
            }

            prompt = <TextInput> {
                width: Fill,
                height: 200,

                empty_message: "Enter your prompt here"
                draw_bg: {
                    color: #ddd
                }
                draw_text: {
                    text_style:<REGULAR_FONT>{font_size: 10},
                    fn get_color(self) -> vec4 {
                        return #555
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
                        sdf.fill(mix(#e99, #d99, self.focus));
                        return sdf.result
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatScreen {
    #[deref]
    view: View,

    #[rust]
    loaded: bool,

    #[rust(true)]
    enabled: bool
}

impl Widget for ChatScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>();
        let chat_history = &store.chat_history;
        let chats_count = chat_history.len();

        while let Some(view_item) = self.view.draw_walk(cx, &mut Scope::empty(), walk).step(){
            if let Some(mut list) = view_item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, chats_count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let item = list.item(cx, item_id, live_id!(ChatLine)).unwrap();

                    if item_id < chats_count {
                        let model_data = &chat_history[item_id];
                        item.label(id!(label)).set_text(model_data);
                        item.draw_all(cx, &mut Scope::with_data(&mut model_data.clone()));
                    }
                }
            }
        }

        DrawStep::done()
    }
}

impl WidgetMatchEvent for ChatScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if !self.loaded && self.button(id!(load_button)).clicked(&actions) {
            self.loaded = true;
            self.view(id!(main)).set_visible(true);

            let store = scope.data.get_mut::<Store>();
            store.load_model();

            self.button(id!(load_button)).set_text("Model loaded");
            self.redraw(cx);
        }

        if self.enabled {
            for action in actions.iter() {
                match action.as_widget_action().cast() {
                    TextInputAction::Return(prompt) => {
                        let store = scope.data.get_mut::<Store>();
                        store.send_chat(prompt.clone());
                        self.redraw(cx);
                    }
                    _ => {}
                }
            }
        }
    }
}