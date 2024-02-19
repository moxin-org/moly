use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;

    SlidePanelModal = {{SlidePanelModal}} {
        //visible: false,

        width: Fill,
        height: Fill,
        flow: Overlay,

        panel = <SlidePanel> {
            width: 923,
            height: Fill,

            cursor: Default,

            side: Right,
            closed: 1.0,

            animator: {
                closed = {
                    default: on
                }
            }
        }
    }
}

#[derive(Default, PartialEq)]
enum SlidePanelState {
    Open,
    Closing,
    #[default] Closed,
}


#[derive(Live, LiveHook, Widget)]
pub struct SlidePanelModal {
    #[deref]
    view: View,

    #[rust]
    state: SlidePanelState,
}

impl Widget for SlidePanelModal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let actions = cx.capture_actions(|cx| self.view.handle_event(cx, event, scope));

        match self.state {
            SlidePanelState::Open => {
                // We need to "capture" all touch events if we have an active modal
                match event.hits(cx, self.view.area()) {
                    Hit::FingerDown(_fd) => {
                        if actions.is_empty() {
                            // If we don't have any actions, we should close the modal
                            self.slide_panel(id!(panel)).close(cx);
                            self.state = SlidePanelState::Closing;
                        }
                    },
                    _ => {},
                }
            },
            SlidePanelState::Closing => {
                if !self.slide_panel(id!(panel)).is_animating(cx) {
                    self.state = SlidePanelState::Closed;
                }
            },
            SlidePanelState::Closed => {},
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl SlidePanelModalRef {
    pub fn show(&mut self, cx: &mut Cx) {
        let Some(mut modals) = self.borrow_mut() else { return };
        modals.apply_over(cx, live!{visible: (true)});
        modals.slide_panel(id!(panel)).open(cx);
        modals.state = SlidePanelState::Open;
    }
}
