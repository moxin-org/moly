use crate::protocol::*;
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::moly_kit_theme::*;

    ITEM_HEIGHT = 64.0;

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
            height: 64,
            list = <PortalList> {
                flow: Right,
                // Image = <ItemView> {
                //     width: (ITEM_HEIGHT),
                // }
                File = <ItemView> {
                    width: (ITEM_HEIGHT * 4),
                    padding: {left: 12., right: 8., top: 8., bottom: 8.},
                    spacing: 12.,
                    align: {y: 0.5},
                    icon = <Label> {
                        text: "",
                        draw_text: {
                            color: #000,
                            text_style: <THEME_FONT_ICONS>{font_size: 28}
                        }
                    }
                    <View> {
                        flow: Down,
                        align: {y: 0.5},
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
}

// Note: Makepad widget macro doesn't let me use `pub(crate)` on the widget struct.
#[derive(Live, Widget, LiveHook)]
pub struct AttachmentList {
    #[deref]
    deref: View,

    // Note: The macro is not letting me use `pub(crate)`.
    #[rust]
    pub attachments: Vec<Attachment>,
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
                    let icon = item.label(id!(icon));
                    let kind = item.label(id!(kind));
                    let title = item.label(id!(title));

                    if attachment.is_available() {
                        if attachment.is_image() {
                            icon.set_text(cx, "\u{f03e}");
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
                            me.attachments.remove(index);
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
