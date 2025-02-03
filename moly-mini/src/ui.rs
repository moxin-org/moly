use makepad_widgets::*;

live_design!(
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::demo_chat::*;

    pub Ui = {{Ui}} <Window> {
        align: {x: 0.5, y: 0.5}
        pass: { clear_color: #fff }

        // caption_bar = {
        //     caption_label = {
        //         // remove the default label
        //         label = <Label> {}
        //         <View> {
        //             width: Fill,
        //             align: {x: 0.5, y: 0.5},
        //             <Label> {
        //                 text: "moly-mini"
        //                 draw_text: {
        //                     color: #000
        //                 }
        //             }
        //         }
        //     }

        //     visible: true,
        // }

        body = <View> {
            <DemoChat> {}
            <DemoChat> {}
        }
    }
);

#[derive(Live, Widget)]
pub struct Ui {
    #[deref]
    deref: Window,
}

impl Widget for Ui {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
    }
}

impl LiveHook for Ui {}
