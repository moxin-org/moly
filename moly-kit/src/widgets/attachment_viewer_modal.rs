use makepad_widgets::*;

use crate::protocol::*;
use crate::utils::events::EventExt;
use crate::widgets::moly_modal::{MolyModalRef, MolyModalWidgetExt};

live_design! {
    use link::theme::*;
    use link::widgets::*;

    use crate::widgets::moly_modal::*;
    use crate::widgets::async_view::*;

    pub AttachmentViewerModal = {{AttachmentViewerModal}} {
        flow: Overlay,
        width: 0,
        height: 0,
        modal = <MolyModal> {
            content: {
                flow: Down,
                width: Fill,
                height: Fill,
                <View> {
                    height: Fit,
                    align: {x: 1},
                    padding: {left: 6, right: 6, top: 4, bottom: 4},
                    close = <Button> {
                        text: "X",
                    }
                }
                image = <Image> {
                    width: Fill,
                    height: Fill,
                    fit: Smallest,
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

        if self.button(id!(close)).clicked(event.actions()) {
            self.close(cx)
        }
    }
}

impl AttachmentViewerModal {
    pub fn open(&mut self, cx: &mut Cx, attachment: &Attachment) {
        self.modal_ref().open(cx);
        const IMG: &[u8] = include_bytes!("../../../packaging/Moly macOS dmg background.png");
        let image = self.image(id!(image));
        image.load_png_from_data(cx, IMG).unwrap();
    }

    pub fn close(&mut self, cx: &mut Cx) {
        eprintln!("Closing modal");
        self.modal_ref().close(cx);
    }

    fn modal_ref(&self) -> MolyModalRef {
        self.moly_modal(id!(modal))
    }
}

impl AttachmentViewerModalRef {
    /// Immutable access to the underlying [`AttachmentViewerModal`].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read(&self) -> std::cell::Ref<AttachmentViewerModal> {
        self.borrow().unwrap()
    }

    /// Mutable access to the underlying [`AttachmentViewerModal`].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write(&mut self) -> std::cell::RefMut<AttachmentViewerModal> {
        self.borrow_mut().unwrap()
    }
}
