use crate::{protocol::*, widgets::attachment_view::AttachmentViewWidgetRefExt};
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::moly_kit_theme::*;

    use crate::widgets::attachment_view::*;

    ITEM_HEIGHT = 200.0;
    ITEM_WIDTH = (ITEM_HEIGHT);

    DENSE_ITEM_HEIGHT = 64.0;
    DENSE_ITEM_WIDTH = (DENSE_ITEM_HEIGHT * 4.0);


    ItemView = {{ItemView}} <RoundedView> {
        height: (ITEM_HEIGHT),
        margin: {right: 4},
        draw_bg: {
            // color: #c00,
            border_radius: 8.0,
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
            height: (ITEM_HEIGHT),
            list = <PortalList> {
                flow: Right,
                // Image = <ItemView> {
                //     width: (ITEM_HEIGHT),
                // }
                File = <ItemView> {
                    flow: Down,
                    height: (ITEM_HEIGHT),
                    width: (ITEM_WIDTH),
                    padding: {left: 12., right: 12., top: 16., bottom: 16.},
                    spacing: 12.,
                    align: {y: 0.5},
                    icon_wrapper = <View> {
                        visible: false,
                        align: {x: 0.5, y: 0.5},
                        icon = <Label> {
                            text: "",
                            draw_text: {
                                color: #000,
                                text_style: <THEME_FONT_ICONS>{font_size: 28}
                            }
                        }
                    }
                    image_wrapper = <View> {
                        image = <AttachmentView> {
                            width: Fill,
                            height: Fill,
                        }
                    }
                    <View> {
                        flow: Down,
                        height: Fit,
                        spacing: 2,
                        title = <Label> {
                            text: "document.pdf",
                            draw_text: {
                                color: #000,
                                text_style: <THEME_FONT_BOLD>{font_size: 11}
                            }
                        }
                        kind = <Label> {
                            text: "PDF",
                            draw_text: {
                                color: #000,
                                text_style: {font_size: 10}
                            }
                        }
                    }
                }
            }
        }
    }

    pub DenseAttachmentList = <AttachmentList> {
        wrapper = {
            height: (DENSE_ITEM_HEIGHT),
            list = {
                File = {
                    flow: Right,
                    height: (DENSE_ITEM_HEIGHT),
                    width: (DENSE_ITEM_WIDTH),
                    padding: {left: 12., right: 8., top: 8., bottom: 8.},
                    icon_wrapper = {
                        width: Fit,
                        align: {y: 0.5},
                    }
                    image_wrapper = {
                        width: 48.0,
                        height: 48.0,
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
                    let icon_wrapper = item.view(id!(icon_wrapper));
                    let icon = item.label(id!(icon));
                    let image_wrapper = item.view(id!(image_wrapper));
                    let mut image = item.attachment_view(id!(image));
                    let kind = item.label(id!(kind));
                    let title = item.label(id!(title));

                    icon_wrapper.set_visible(cx, true);
                    image_wrapper.set_visible(cx, false);

                    if attachment.is_available() {
                        if attachment.is_image() {
                            icon.set_text(cx, "\u{f03e}");

                            if attachment.content_type.as_deref() == Some("image/png") {
                                image.write().set_attachment(cx, attachment.clone());
                                icon_wrapper.set_visible(cx, false);
                                image_wrapper.set_visible(cx, true);
                            }
                        } else {
                            icon.set_text(cx, "\u{f15b}");
                        }

                        kind.set_text(
                            cx,
                            attachment
                                .content_type
                                .as_deref()
                                .map(|s| s.split('/').last().unwrap_or_default().to_uppercase())
                                .unwrap_or_default()
                                .as_str(),
                        );
                    } else {
                        icon.set_text(cx, "\u{f127}");
                        kind.set_text(cx, "Unavailable");
                    }

                    title.set_text(cx, &attachment.name);

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
