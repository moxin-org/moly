use makepad_widgets::*;

use crate::protocol::*;
use crate::utils::makepad::events::EventExt;
use crate::widgets::attachment_view::AttachmentViewWidgetExt;
use crate::widgets::moly_modal::{MolyModalRef, MolyModalWidgetExt};

live_design! {
    use link::theme::*;
    use link::widgets::*;

    use crate::widgets::moly_modal::*;
    use crate::widgets::attachment_view::*;

    pub AttachmentViewerModal = {{AttachmentViewerModal}} {
        flow: Overlay,
        width: 0,
        height: 0,
        modal = <MolyModal> {
            content: {
                // TODO: Using fill in the content breaks the underlying modal backdrop
                // close on click behavior.
                width: Fill,
                height: Fill,
                <View> {
                    flow: Down,
                    padding: 16,
                    spacing: 16,
                    <View> {
                        height: Fit,
                        align: {x: 1},
                        spacing: 4,
                        save = <Button> {text: "Save"}
                        close = <Button> {text: "X"}
                    }
                    attachment = <AttachmentView> {}
                }
            }
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct AttachmentViewerModal {
    #[deref]
    deref: View,
}

impl Widget for AttachmentViewerModal {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);

        if self.button(ids!(modal.save)).clicked(event.actions()) {
            self.attachment_view(ids!(attachment))
                .borrow()
                .unwrap()
                .get_attachment()
                .save();
        }

        if self.button(ids!(modal.close)).clicked(event.actions()) {
            self.close(cx)
        }
    }
}

impl AttachmentViewerModal {
    pub fn open(&mut self, cx: &mut Cx, attachment: Attachment) {
        self.modal_ref().open(cx);
        self.attachment_view(ids!(attachment))
            .borrow_mut()
            .unwrap()
            .set_attachment(cx, attachment);
    }

    pub fn close(&mut self, cx: &mut Cx) {
        self.modal_ref().close(cx);
    }

    fn modal_ref(&self) -> MolyModalRef {
        self.moly_modal(ids!(modal))
    }
}
