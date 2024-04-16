use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;

    ModalView = {{ModalView}} {
        width: Fill
        height: Fill
        flow: Overlay
        align: {x: 0.5, y: 0.5}

        bg_view = <View> {
            width: Fill
            height: Fill
            show_bg: true
            draw_bg: {
                fn pixel(self) -> vec4 {
                    return vec4(0., 0., 0., 0.7)
                }
            }
        }

        content = <View> {
            flow: Overlay
            width: Fit
            height: Fit
            // This assures that the hit events gets consummed when clicking the content, so it closes when clicking outside of it.
            cursor: Arrow
        }
    }

    Modal = {{Modal}} {
        width: Fill
        height: Fill

        flow: Right
    }
}

#[derive(Live, LiveHook, LiveRegisterWidget, WidgetRef)]
pub struct ModalView {
    #[deref]
    view: View,
}

impl Widget for ModalView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let widget_uid = self.widget_uid();

        self.view(id!(content)).handle_event(cx, event, scope);

        match event.hits(cx, self.view(id!(bg_view)).area()) {
            Hit::FingerUp(_fe) => {
                cx.widget_action(widget_uid, &scope.path, ModalAction::CloseModal);
            }
            _ => (),
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view
            .draw_walk(cx, scope, walk.with_abs_pos(DVec2 { x: 0., y: 0. }))
    }
}

impl WidgetNode for ModalView {
    fn walk(&mut self, cx: &mut Cx) -> Walk {
        self.view.walk(cx)
    }

    fn redraw(&mut self, cx: &mut Cx) {
        self.view.redraw(cx);
    }

    fn find_widgets(&mut self, path: &[LiveId], cached: WidgetCache, results: &mut WidgetSet) {
        self.view.find_widgets(path, cached, results);
    }
}

impl ModalView {
    pub fn show(&mut self, cx: &mut Cx) {
        self.apply_over(cx, live! {visible: true});
        self.redraw(cx);
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        self.apply_over(cx, live! {visible: false});
        self.redraw(cx);
    }

    pub fn is_showing(&self) -> bool {
        self.view.is_visible()
    }
}

impl ModalViewRef {
    pub fn show(&mut self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show(cx);
        }
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.hide(cx)
        }
    }

    pub fn is_showing(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.is_showing()
        } else {
            false
        }
    }
}

#[derive(Clone, DefaultNone, Eq, Hash, PartialEq, Debug)]
pub enum ModalAction {
    None,
    ShowModalView(LiveId),
    CloseModal,
}

#[derive(Default)]
enum ActiveModalView {
    #[default]
    None,
    Active(LiveId),
}

#[derive(Live, LiveRegisterWidget, WidgetRef)]
pub struct Modal {
    #[deref]
    view: View,

    #[rust]
    active_modal_view: ActiveModalView,
}

impl Widget for Modal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if let Some(widget_ref) = self.get_active_modal_view(cx) {
            widget_ref.handle_event(cx, event, scope);
        }

        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if let Some(widget_ref) = self.get_active_modal_view(cx) {
            widget_ref.draw_walk(cx, scope, walk)?;
        }
        DrawStep::done()
    }
}

impl LiveHook for Modal {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {}
}

impl WidgetNode for Modal {
    fn walk(&mut self, cx: &mut Cx) -> Walk {
        self.view.walk(cx)
    }

    fn redraw(&mut self, cx: &mut Cx) {
        if let Some(widget_ref) = self.get_active_modal_view(cx) {
            widget_ref.redraw(cx);
        }
    }

    fn find_widgets(&mut self, path: &[LiveId], cached: WidgetCache, results: &mut WidgetSet) {
        self.view.find_widgets(path, cached, results);
    }
}

impl WidgetMatchEvent for Modal {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for action in actions {
            match action.as_widget_action().cast::<ModalAction>() {
                ModalAction::ShowModalView(modal_view_id) => {
                    if let Err(err) = self.show_modal_view_by_id(cx, modal_view_id) {
                        error!("{err}")
                    }
                }
                ModalAction::CloseModal => {
                    self.close(cx);
                }
                ModalAction::None => {}
            }
        }
    }
}

impl Modal {
    fn get_active_modal_view(&mut self, _cx: &mut Cx) -> Option<ModalViewRef> {
        match self.active_modal_view {
            ActiveModalView::None => None,
            ActiveModalView::Active(modal_view_id) => {
                let modal_view_ref = self.modal_view(&[modal_view_id]);

                if modal_view_ref.is_showing() {
                    Some(modal_view_ref)
                } else {
                    None
                }
            }
        }
    }

    pub fn show_modal_view_by_id(
        &mut self,
        cx: &mut Cx,
        modal_view_id: LiveId,
    ) -> Result<(), String> {
        let mut modal_view_ref = self.modal_view(&[modal_view_id]);

        if modal_view_ref.is_empty() {
            return Err(format!("ModalView with id '{modal_view_id}' not found"));
        }

        if let Some(mut current_active_modal_view_ref) = self.get_active_modal_view(cx) {
            current_active_modal_view_ref.hide(cx);
        }

        modal_view_ref.show(cx);
        self.active_modal_view = ActiveModalView::Active(modal_view_id);

        self.redraw(cx);
        Ok(())
    }

    pub fn close(&mut self, cx: &mut Cx) {
        if let Some(mut current_active_modal_view_ref) = self.get_active_modal_view(cx) {
            current_active_modal_view_ref.hide(cx);
        }

        self.apply_over(cx, live! {visible: false});
    }
}

impl ModalRef {
    pub fn show_modal_view_by_id(
        &mut self,
        cx: &mut Cx,
        stack_view_id: LiveId,
    ) -> Result<(), String> {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show_modal_view_by_id(cx, stack_view_id)
        } else {
            Err("Widget not found in the document".to_string())
        }
    }

    pub fn close(&mut self, cx: &mut Cx) -> Result<(), String> {
        if let Some(mut inner) = self.borrow_mut() {
            inner.close(cx);
            Ok(())
        } else {
            Err("Widget not found in the document".to_string())
        }
    }
}
