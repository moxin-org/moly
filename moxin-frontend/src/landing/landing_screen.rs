use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::landing::model_list::ModelList;

    LandingScreen = {{LandingScreen}} {
        width: Fill,
        height: Fill,
        flow: Overlay,

        <View> {
            width: Fill,
            height: Fill,
            flow: Down,
            margin: 50,
            spacing: 30,

            <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 20},
                    color: #000
                }
                text: "Explore"
            }

            <ModelList> {}
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct LandingScreen {
    #[deref]
    view: View
}

impl Widget for LandingScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
            self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}