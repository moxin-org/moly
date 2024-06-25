use std::any::Any;
use std::fmt::Debug;

use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;

    PortalView = {{PortalView}} {
        width: Fill
        height: Fill
        flow: Overlay
    }

    Portal = {{Portal}} {
        width: Fill
        height: Fill

        flow: Right
    }
}

#[derive(Live, LiveHook, LiveRegisterWidget, WidgetRef)]
pub struct PortalView {
    #[deref]
    view: View,
}

impl Widget for PortalView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view
            .draw_walk(cx, scope, walk.with_abs_pos(DVec2 { x: 0., y: 0. }))
    }
}

impl WidgetNode for PortalView {
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

impl PortalView {
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

impl PortalViewRef {
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

#[derive(Clone, DefaultNone, Eq, PartialEq, Debug)]
pub enum PortalAction {
    None,
    ShowPortalView(LiveId),
    // ShowPortalViewWithProps(Box<PortalActionWithProps>),
    Close,
}

// #[derive(Debug)]
// pub struct PortalActionWithProps {
//     id: LiveId,
//     data: Box<dyn Any>,
// }
//
// impl PortalActionWithProps {
//     pub fn new<T: 'static>(id: LiveId, data: T) -> Self {
//         Self {
//             id,
//             data: Box::new(data),
//         }
//     }
//
//     pub fn get_data<T: 'static>(&self) -> Option<&T> {
//         self.data.downcast_ref::<T>()
//     }
// }
//
// impl PortalAction {
//     pub fn new_show_portal_view_with_props<T: 'static>(id: LiveId, data: T) -> Self {
//         PortalAction::ShowPortalViewWithProps(Box::new(PortalActionWithProps::new(id, data)))
//     }
// }
//
// impl Clone for PortalActionWithProps {
//     fn clone(&self) -> Self {
//         Self {
//             id: self.id.clone(),
//             data: self.data.clone(),
//         }
//     }
// }
//
// impl PartialEq for PortalActionWithProps {
//     fn eq(&self, other: &Self) -> bool {
//         self.id == other.id && self.data.type_id() == other.data.type_id()
//     }
// }
//
// impl Eq for PortalActionWithProps {}
//
// trait CloneAny {
//     fn clone_any(&self) -> Box<dyn Any>;
// }
//
// impl<T: 'static + Clone> CloneAny for T {
//     fn clone_any(&self) -> Box<dyn Any> {
//         Box::new(self.clone())
//     }
// }

#[derive(Default)]
enum ActivePortalView {
    #[default]
    None,
    Active(LiveId),
}

#[derive(Live, LiveRegisterWidget, WidgetRef)]
pub struct Portal {
    #[deref]
    view: View,

    #[rust]
    active_portal_view: ActivePortalView,
}

impl Widget for Portal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if let Some(widget_ref) = self.get_active_portal_view(cx) {
            widget_ref.handle_event(cx, event, scope);
        }

        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if let Some(widget_ref) = self.get_active_portal_view(cx) {
            widget_ref.draw_walk(cx, scope, walk)?;
        }

        DrawStep::done()
    }
}

impl LiveHook for Portal {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {}
}

impl WidgetNode for Portal {
    fn walk(&mut self, cx: &mut Cx) -> Walk {
        self.view.walk(cx)
    }

    fn redraw(&mut self, cx: &mut Cx) {
        if let Some(widget_ref) = self.get_active_portal_view(cx) {
            widget_ref.redraw(cx);
        }
    }

    fn find_widgets(&mut self, path: &[LiveId], cached: WidgetCache, results: &mut WidgetSet) {
        self.view.find_widgets(path, cached, results);
    }
}

impl WidgetMatchEvent for Portal {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for action in actions {
            match action.as_widget_action().cast::<PortalAction>() {
                PortalAction::ShowPortalView(portal_view_id) => {
                    if let Err(err) = self.show_portal_view_by_id(cx, portal_view_id) {
                        error!("{err}")
                    }
                }
                PortalAction::Close => {
                    self.close(cx);
                }
                PortalAction::None => {}
            }
        }
    }
}

impl Portal {
    fn get_active_portal_view(&mut self, _cx: &mut Cx) -> Option<PortalViewRef> {
        match self.active_portal_view {
            ActivePortalView::None => None,
            ActivePortalView::Active(portal_view_id) => {
                let portal_view_ref = self.portal_view(&[portal_view_id]);

                if portal_view_ref.is_showing() {
                    Some(portal_view_ref)
                } else {
                    None
                }
            }
        }
    }

    pub fn show_portal_view_by_id(
        &mut self,
        cx: &mut Cx,
        portal_view_id: LiveId,
    ) -> Result<(), &'static str> {
        let mut portal_view_ref = self.portal_view(&[portal_view_id]);

        if portal_view_ref.is_empty() {
            return Err("PortalView not found");
        }

        if let Some(mut current_active_portal_view_ref) = self.get_active_portal_view(cx) {
            current_active_portal_view_ref.hide(cx);
        }

        portal_view_ref.show(cx);
        self.active_portal_view = ActivePortalView::Active(portal_view_id);

        self.redraw(cx);
        Ok(())
    }

    // pub fn show_portal_view_by_id_with_props<T>(
    //     &mut self,
    //     cx: &mut Cx,
    //     portal_view_id: LiveId,
    //     props: T,
    // ) -> Result<(), &'static str> {
    //     self.show_portal_view_by_id(cx, portal_view_id)
    // }

    pub fn close(&mut self, cx: &mut Cx) {
        if let Some(mut current_active_portal_view_ref) = self.get_active_portal_view(cx) {
            current_active_portal_view_ref.hide(cx);
        }

        self.apply_over(cx, live! {visible: false});
    }
}

#[allow(dead_code)]
impl PortalRef {
    pub fn show_portal_view_by_id(
        &mut self,
        cx: &mut Cx,
        stack_view_id: LiveId,
    ) -> Result<(), &'static str> {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show_portal_view_by_id(cx, stack_view_id)
        } else {
            Err("Widget not found in the document")
        }
    }

    pub fn close(&mut self, cx: &mut Cx) -> Result<(), &'static str> {
        if let Some(mut inner) = self.borrow_mut() {
            inner.close(cx);
            Ok(())
        } else {
            Err("Widget not found in the document")
        }
    }
}
