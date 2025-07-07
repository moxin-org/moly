use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub GalleryModal = {{GalleryModal}} {
        width: 300,
        height: 300,
        <View> {
            width: 300,
            height: 300,
            show_bg: true
            draw_bg: {
                color: #f00,
            }
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct GalleryModal {
    #[deref]
    deref: View,
}

impl Widget for GalleryModal {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // using abs pos shows nothing, but withouit it shows the red square
        self.deref
            .draw_walk(cx, scope, walk.with_abs_pos(DVec2 { x: 0., y: 0. }))
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope)
    }
}
