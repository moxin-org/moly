use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    CButton = {{CButton}} <RoundedView> {
        align: {x: 0.5, y: 0.5}
        flow: Right
        width: Fit, height: Fit
        padding: { top: 15, bottom: 15, left: 8, right: 13}
        spacing: 10
        draw_bg: {
            radius: 2.0,
            color: #fff,
            border_width: 1.0,
            border_color: #000,
        }

        icon = <Icon> {
            draw_icon: {
                fn get_color(self) -> vec4 {
                    return #000;
                }
            }
            icon_walk: {width: 12, height: 12}
        }

        label = <Label> {}
    }
);

/// An attempt of a custom button, sharable with the rest of the app.
#[derive(Widget, Live, LiveHook)]
pub struct CButton {
    #[deref]
    deref: View,
}

impl Widget for CButton {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        match event.hits(cx, self.area()) {
            Hit::FingerDown(_) => {
                cx.set_key_focus(self.deref.area());
            }
            Hit::FingerUp(up) => {
                if up.was_tap() {
                    cx.widget_action(self.widget_uid(), &scope.path, CButtonAction::Tapped);
                }
            }
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
            }
            Hit::FingerHoverOut(_) => {
                cx.set_cursor(MouseCursor::Arrow);
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum CButtonAction {
    Tapped,
    /// Needed to support `.cast()`.
    None,
}

impl CButton {
    pub fn tapped(&self, actions: &Actions) -> bool {
        matches!(
            actions.find_widget_action(self.widget_uid()).cast(),
            CButtonAction::Tapped
        )
    }
}

impl CButtonRef {
    pub fn tapped(&self, actions: &Actions) -> bool {
        if let Some(widget) = self.borrow() {
            widget.tapped(actions)
        } else {
            false
        }
    }
}
