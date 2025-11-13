//! Copy of the original modal from the main Moly app which draws its content
//! over the whole app (from its root).

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    pub MolyModal = {{MolyModal}} {
        width: Fill
        height: Fill
        flow: Overlay
        align: {x: 0.5, y: 0.5}

        draw_bg: {
            fn pixel(self) -> vec4 {
                return vec4(0., 0., 0., 0.0)
            }
        }

        bg_view: <View> {
            width: Fill
            height: Fill
            show_bg: true
            draw_bg: {
                fn pixel(self) -> vec4 {
                    return vec4(0., 0., 0., 0.7)
                }
            }
        }

        content: <View> {
            flow: Overlay
            width: Fit
            height: Fit
        }
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum MolyModalAction {
    None,
    Dismissed,
}

#[derive(Live, Widget)]
pub struct MolyModal {
    #[live]
    #[find]
    content: View,
    #[live]
    #[area]
    bg_view: View,

    #[redraw]
    #[rust(DrawList2d::new(cx))]
    draw_list: DrawList2d,

    #[live]
    draw_bg: DrawQuad,
    #[layout]
    layout: Layout,
    #[walk]
    walk: Walk,

    #[live(true)]
    dismiss_on_focus_lost: bool,

    #[rust]
    opened: bool,
}

impl LiveHook for MolyModal {
    fn after_apply(&mut self, cx: &mut Cx, _apply: &mut Apply, _index: usize, _nodes: &[LiveNode]) {
        self.draw_list.redraw(cx);
    }
}

impl Widget for MolyModal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.opened {
            return;
        }

        // When passing down events we need to suspend the sweep lock
        // because regular View instances won't respond to events if the sweep lock is active.
        cx.sweep_unlock(self.draw_bg.area());
        self.content.handle_event(cx, event, scope);
        cx.sweep_lock(self.draw_bg.area());

        if self.dismiss_on_focus_lost {
            // Check if there was a click outside of the content (bg), then close if true.
            let content_rec = self.content.area().rect(cx);
            if let Hit::FingerUp(fe) =
                event.hits_with_sweep_area(cx, self.draw_bg.area(), self.draw_bg.area())
            {
                if !content_rec.contains(fe.abs) {
                    let widget_uid = self.content.widget_uid();
                    cx.widget_action(widget_uid, &scope.path, MolyModalAction::Dismissed);
                    self.close(cx);
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_list.begin_overlay_reuse(cx);

        cx.begin_root_turtle_for_pass(self.layout);
        self.draw_bg.begin(cx, self.walk, self.layout);

        if self.opened {
            let _ = self
                .bg_view
                .draw_walk(cx, scope, walk.with_abs_pos(DVec2 { x: 0., y: 0. }));
            let _ = self.content.draw_all(cx, scope);
        }

        self.draw_bg.end(cx);

        cx.end_pass_sized_turtle();
        self.draw_list.end(cx);

        DrawStep::done()
    }
}

impl MolyModal {
    #[deprecated(note = "Use open_as_dialog or open_as_popup instead")]
    pub fn open(&mut self, cx: &mut Cx) {
        self.opened = true;
        self.draw_bg.redraw(cx);
        cx.sweep_lock(self.draw_bg.area());
    }

    pub fn open_as_dialog(&mut self, cx: &mut Cx) {
        self.apply_over(
            cx,
            live! {
                align: {x: 0.5, y: 0.5}
                content: {
                    margin: 0,
                }
                bg_view: {
                    visible: true
                }
            },
        );

        #[allow(deprecated)]
        self.open(cx);
    }

    pub fn open_as_popup(&mut self, cx: &mut Cx, pos: Vec2) {
        self.apply_over(
            cx,
            live! {
                align: {x: 0.0, y: 0.0}
                content: {
                    margin: {left: (pos.x), top: (pos.y) }
                }
                bg_view: {
                    visible: false
                }
            },
        );

        #[allow(deprecated)]
        self.open(cx);
    }

    pub fn close(&mut self, cx: &mut Cx) {
        self.opened = false;
        self.draw_bg.redraw(cx);
        cx.sweep_unlock(self.draw_bg.area())
    }

    pub fn dismissed(&self, actions: &Actions) -> bool {
        matches!(
            actions.find_widget_action(self.widget_uid()).cast(),
            MolyModalAction::Dismissed
        )
    }
}

impl MolyModalRef {
    #[deprecated(note = "Use open_as_dialog or open_as_popup instead")]
    pub fn open(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.open(cx);
        }
    }

    pub fn open_as_dialog(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.open_as_dialog(cx);
        }
    }

    pub fn open_as_popup(&self, cx: &mut Cx, pos: Vec2) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.open_as_popup(cx, pos);
        }
    }

    pub fn close(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.close(cx);
        }
    }

    pub fn dismissed(&self, actions: &Actions) -> bool {
        if let Some(inner) = self.borrow() {
            inner.dismissed(actions)
        } else {
            false
        }
    }
}
