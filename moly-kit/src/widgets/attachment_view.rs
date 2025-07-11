use crate::{
    protocol::*,
    utils::asynchronous::{AbortOnDropHandle, abort_on_drop, spawn},
    widgets::image_contain::{ImageContainRef, ImageContainWidgetExt},
};
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    use crate::widgets::image_contain::*;

    pub AttachmentView = {{AttachmentView}} {
        icon_wrapper = <View> {
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
            image = <ImageContain> {}
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct AttachmentView {
    #[deref]
    deref: View,

    #[rust]
    attachment: Attachment,

    #[rust]
    abort_on_drop: Option<AbortOnDropHandle>,
}

impl Widget for AttachmentView {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope)
    }
}

impl AttachmentView {
    pub fn set_attachment(&mut self, cx: &mut Cx, attachment: Attachment) {
        // Only trigger stuff if attachment has changed.
        if self.attachment != attachment {
            // Preserve for future comparisons.
            self.attachment = attachment;

            let icon = self.label(id!(icon));

            self.icon_wrapper_ref().set_visible(cx, true);
            self.image_wrapper_ref().set_visible(cx, false);

            if self.attachment.is_available() {
                if self.attachment.is_image() {
                    icon.set_text(cx, "\u{f03e}");
                    self.try_load();
                } else {
                    icon.set_text(cx, "\u{f15b}");
                }
            } else {
                icon.set_text(cx, "\u{f127}");
            }
        }
    }

    fn image_ref(&self) -> ImageContainRef {
        self.image_contain(id!(image))
    }

    fn image_wrapper_ref(&self) -> ViewRef {
        self.view(id!(image_wrapper))
    }

    fn icon_wrapper_ref(&self) -> ViewRef {
        self.view(id!(icon_wrapper))
    }

    fn try_load(&mut self) {
        if self.attachment.content_type.as_deref() != Some("image/png") {
            return;
        }

        let ui = self.ui_runner();
        let attachment = self.attachment.clone();

        let future = async move {
            let Ok(content) = attachment.read().await else {
                error!(
                    "Failed to read attachment content of type {} for {}",
                    attachment.content_type.as_deref().unwrap_or("unknown"),
                    attachment.name
                );
                return;
            };

            ui.defer_with_redraw(move |me, cx, _| {
                if let Err(e) = me.image_ref().write().load_png(cx, &content) {
                    error!(
                        "Failed to load attachment {} as PNG: {}",
                        attachment.name, e
                    );
                }

                me.icon_wrapper_ref().set_visible(cx, false);
                me.image_wrapper_ref().set_visible(cx, true);
            });
        };

        let (future, abort_on_drop) = abort_on_drop(future);
        self.abort_on_drop = Some(abort_on_drop);
        spawn(async move {
            let _ = future.await;
        });
    }
}

impl AttachmentViewRef {
    /// Immutable access to the underlying [`AttachmentView`].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read(&self) -> std::cell::Ref<AttachmentView> {
        self.borrow().unwrap()
    }

    /// Mutable access to the underlying [`AttachmentView`].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write(&mut self) -> std::cell::RefMut<AttachmentView> {
        self.borrow_mut().unwrap()
    }
}
