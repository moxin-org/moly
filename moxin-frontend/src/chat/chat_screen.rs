use makepad_widgets::*;
use crate::data::store::Store;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::chat::chat_panel::ChatPanel;

    ChatScreen = {{ChatScreen}} {
        width: Fill,
        height: Fill,
        margin: 50,
        spacing: 30,

        <View> {
            width: 200,
            height: Fill,
        }
    
        chat_panel = <ChatPanel> {
            width: Fill,
            height: Fill,
        }

        <View> {
            width: 200,
            height: Fill,
        }   
    }    
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatScreen {
    #[deref]
    view: View,
}

impl Widget for ChatScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}