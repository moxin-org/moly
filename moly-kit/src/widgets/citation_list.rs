use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    use crate::widgets::citation::*;

    pub CitationList = {{CitationList}} {
        width: Fill,
        height: Fit,
        list = <PortalList> {
            flow: Right,
            width: Fill,
            // Fit doesn't work here.
            height: 64,
            Citation = <Citation> {
                // spacing on parent doesn't work
                margin: {right: 8},
            }
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct CitationList {
    #[deref]
    deref: View,
}

impl Widget for CitationList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let list_uid = self.portal_list(id!(list)).widget_uid();
        while let Some(widget) = self.deref.draw_walk(cx, scope, walk).step() {
            if widget.widget_uid() == list_uid {
                self.draw_list(cx, &mut *widget.as_portal_list().borrow_mut().unwrap());
            }
        }

        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope)
    }
}

impl CitationList {
    fn draw_list(&mut self, cx: &mut Cx2d, list: &mut PortalList) {
        list.set_item_range(cx, 0, 10);
        while let Some(index) = list.next_visible_item(cx) {
            if index >= 10 {
                continue;
            }

            let item = list.item(cx, index, live_id!(Citation));
            item.draw_all(cx, &mut Scope::empty());
        }
    }
}
