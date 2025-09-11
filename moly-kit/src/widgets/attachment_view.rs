use crate::{
    protocol::*,
    utils::{
        asynchronous::{AbortOnDropHandle, abort_on_drop, spawn},
        makepad::hex_rgb_color,
    },
    widgets::image_view::{ImageViewRef, ImageViewWidgetExt},
};
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::moly_kit_theme::*;

    use crate::widgets::image_view::*;

    pub AttachmentView = {{AttachmentView}} {
        flow: Overlay,
        icon_wrapper = <View> {
            flow: Down,
            align: {x: 0.5, y: 0.5},
            spacing: 2,
            icon = <Label> {
                text: "",
                draw_text: {
                    color: #000,
                    text_style: <THEME_FONT_ICONS>{font_size: 28}
                }
            }
            title = <Label> {
                text: "document.pdf",
                draw_text: {
                    color: #000,
                    text_style: {font_size: 11}
                }
            }
        }

        image_wrapper = <View> {
            image = <ImageView> {contain: true}
        }

        tag_wrapper = <View> {
            visible: false,
            align: {x: 1}
            tag_bg = <RoundedView> {
                width: Fit,
                height: Fit,
                margin: 8,
                draw_bg: {
                    color: #000,
                    border_radius: 4.0,
                }
                tag_label = <Label> {
                    text: "PDF",
                    padding: {left: 2, right: 2, top: 1.6, bottom: 1.6},
                    draw_text: {
                        color: #fff,
                        text_style: <THEME_FONT_BOLD>{font_size: 8}
                    }
                }
            }
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
            let tag_label = self.label(id!(tag_label));
            let title = self.label(id!(title));

            self.icon_wrapper_ref().set_visible(cx, true);
            self.image_wrapper_ref().set_visible(cx, false);

            tag_label.set_text(
                cx,
                self.attachment
                    .content_type
                    .as_deref()
                    .map(|s| s.split('/').last().unwrap_or_default().to_uppercase())
                    .unwrap_or_default()
                    .as_str(),
            );

            self.tag_bg_ref().apply_over(
                cx,
                live! {
                    draw_bg: {
                        color: (no_preview_color()),
                    }
                },
            );

            title.set_text(cx, &self.attachment.name);

            if self.attachment.is_available() {
                if self.attachment.is_image() {
                    icon.set_text(cx, "\u{f03e}");
                    self.try_load_preview();
                } else {
                    icon.set_text(cx, "\u{f15b}");
                }
            } else {
                icon.set_text(cx, "\u{f127}");
                tag_label.set_text(cx, "Unavailable");
                self.tag_bg_ref().apply_over(
                    cx,
                    live! {
                        draw_bg: {
                            color: (unavailable_color()),
                        }
                    },
                );
            }
        }
    }

    #[allow(unused)]
    pub fn get_texture(&self) -> Option<Texture> {
        self.image_ref().borrow().unwrap().get_texture()
    }

    pub fn get_attachment(&self) -> &Attachment {
        &self.attachment
    }

    fn image_ref(&self) -> ImageViewRef {
        self.image_view(id!(image))
    }

    fn image_wrapper_ref(&self) -> ViewRef {
        self.view(id!(image_wrapper))
    }

    fn icon_wrapper_ref(&self) -> ViewRef {
        self.view(id!(icon_wrapper))
    }

    fn tag_bg_ref(&self) -> ViewRef {
        self.view(id!(tag_bg))
    }

    fn try_load_preview(&mut self) {
        // Not even try if not a supported image.
        if !crate::widgets::image_view::can_load(self.attachment.content_type_or_octet_stream()) {
            return;
        }

        let ui = self.ui_runner();
        let attachment = self.attachment.clone();

        let future = async move {
            let Ok(content) = attachment.read().await else {
                ::log::error!(
                    "Failed to read attachment content of type {} for {}",
                    attachment.content_type_or_octet_stream(),
                    attachment.name
                );
                return;
            };

            ui.defer_with_redraw(move |me, cx, _| {
                if let Err(e) = me.image_ref().borrow_mut().unwrap().load_with_contet_type(
                    cx,
                    &content,
                    attachment.content_type_or_octet_stream(),
                ) {
                    ::log::warn!(
                        "Failed to load attachment {} as {}: {}",
                        attachment.name,
                        attachment.content_type_or_octet_stream(),
                        e
                    );
                }

                me.icon_wrapper_ref().set_visible(cx, false);
                me.image_wrapper_ref().set_visible(cx, true);
                me.tag_bg_ref().apply_over(
                    cx,
                    live! {
                        draw_bg: {
                            color: (preview_color()),
                        }
                    },
                );
            });
        };

        let (future, abort_on_drop) = abort_on_drop(future);
        self.abort_on_drop = Some(abort_on_drop);
        spawn(async move {
            let _ = future.await;
        });
    }
}

/// Red-ish to catch the attention.
fn unavailable_color() -> Vec4 {
    hex_rgb_color(0xec003f)
}

/// Blue-ish color because it's neutral and doesn't really matter.
fn no_preview_color() -> Vec4 {
    hex_rgb_color(0x0084d1)
}

/// Green-ish because it's less common in human pictures.
fn preview_color() -> Vec4 {
    hex_rgb_color(0x00a63e)
}

/// If this widget could generate a preview for the attachment.
pub fn can_preview(attachment: &Attachment) -> bool {
    attachment.is_available()
        && crate::widgets::image_view::can_load(attachment.content_type_or_octet_stream())
}
