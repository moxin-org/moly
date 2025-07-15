use crate::{protocol::*, widgets::attachment_view::AttachmentViewWidgetRefExt};
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::moly_kit_theme::*;

    use crate::widgets::attachment_view::*;

    ITEM_HEIGHT = 200.0;
    ITEM_WIDTH = (ITEM_HEIGHT);
    ITEM_RADIUS = 8.0;

    DENSE_ITEM_HEIGHT = (ITEM_HEIGHT * 0.5);
    DENSE_ITEM_WIDTH = (ITEM_WIDTH * 0.5);
    DENSE_ITEM_RADIUS = (ITEM_RADIUS * 0.75);


    ItemView = {{ItemView}} <RoundedView> {
        height: (ITEM_HEIGHT),
        width: (ITEM_WIDTH),
        margin: {right: 4},
        cursor: Hand,
        draw_bg: {
            border_radius: (ITEM_RADIUS),
            border_color: #D0D5DD,
            border_size: 1.0,
        }
    }

    pub AttachmentList = {{AttachmentList}} {
        height: Fit,
        // The wrapper is just to control visibility. If we put this in the main widget,
        // `draw_walk` will not run at all, making visibility binding harder.
        wrapper = <View> {
            visible: false,
            height: Fit,
            list = <PortalList> {
                flow: Right,
                height: (ITEM_HEIGHT),
                scroll_bar: {bar_size: 0.0}

                File = <ItemView> {
                    preview_wrapper = <CachedRoundedView> {
                        draw_bg: {
                            border_radius: (ITEM_RADIUS),
                        }
                        preview = <AttachmentView> {
                            image_wrapper = {
                                image = {contain: false}
                            }
                            tag_wrapper = {visible: true}
                        }
                    }
                }
            }
        }
    }

    pub DenseAttachmentList = <AttachmentList> {
        wrapper = {
            list = {
                height: (DENSE_ITEM_HEIGHT),
                File = {
                    height: (DENSE_ITEM_HEIGHT),
                    width: (DENSE_ITEM_WIDTH),
                    draw_bg: {
                        border_radius: (DENSE_ITEM_RADIUS),
                    }
                    preview_wrapper = {
                        draw_bg: {
                            border_radius: (DENSE_ITEM_RADIUS),
                        }
                    }
                }
            }
        }
    }
}

// Note: Makepad widget macro doesn't let me use `pub(crate)` on the widget struct.
#[derive(Live, Widget, LiveHook)]
pub struct AttachmentList {
    #[deref]
    deref: View,

    // Note: The macro is not letting me use `pub(crate)`.
    #[rust]
    pub attachments: Vec<Attachment>,

    #[rust]
    pub on_tap: Option<Box<dyn FnMut(&mut AttachmentList, usize) + 'static>>,
}

impl Widget for AttachmentList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view(id!(wrapper))
            .set_visible(cx, !self.attachments.is_empty());

        let attachments_count = self.attachments.len();
        let list = self.portal_list(id!(list));
        while let Some(widget) = self.deref.draw_walk(cx, scope, walk).step() {
            if widget.widget_uid() == list.widget_uid() {
                let mut list = list.borrow_mut().unwrap();
                list.set_item_range(cx, 0, attachments_count);
                while let Some(index) = list.next_visible_item(cx) {
                    if index >= attachments_count {
                        continue;
                    }

                    let attachment = &self.attachments[index];
                    let item = list.item(cx, index, live_id!(File));

                    item.attachment_view(id!(preview))
                        .borrow_mut()
                        .unwrap()
                        .set_attachment(cx, attachment.clone());

                    // Tired of fighthing an event bubbling issue for an internal widget...
                    let ui = self.ui_runner();
                    item.as_item_view().borrow_mut().unwrap().on_tap = Some(Box::new(move || {
                        ui.defer_with_redraw(move |me, _, _| {
                            if let Some(mut on_tap) = me.on_tap.take() {
                                on_tap(me, index);
                                me.on_tap = Some(on_tap);
                            }
                        });
                    }));

                    item.draw_all_unscoped(cx);
                }
            }
        }

        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope)
    }
}

impl AttachmentList {
    pub fn on_tap<F>(&mut self, f: F)
    where
        F: FnMut(&mut AttachmentList, usize) + 'static,
    {
        self.on_tap = Some(Box::new(f));
    }
}

impl AttachmentListRef {
    /// Immutable access to the underlying [[AttachmentList]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read(&self) -> std::cell::Ref<AttachmentList> {
        self.borrow().unwrap()
    }

    /// Mutable access to the underlying [[AttachmentList]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write(&mut self) -> std::cell::RefMut<AttachmentList> {
        self.borrow_mut().unwrap()
    }
}

#[derive(Live, Widget, LiveHook)]
struct ItemView {
    #[deref]
    deref: View,

    #[rust]
    on_tap: Option<Box<dyn FnMut() + 'static>>,
}

impl Widget for ItemView {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
        if let Hit::FingerUp(fu) = event.hits(cx, self.area()) {
            if fu.was_tap() {
                if let Some(on_tap) = &mut self.on_tap {
                    on_tap();
                }
            }
        }
    }
}
