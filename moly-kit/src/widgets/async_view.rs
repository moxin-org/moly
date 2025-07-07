use crate::utils::asynchronous::{PlatformSendFuture, spawn};
use futures::future::AbortHandle;
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub AsyncView = {{AsyncView}} {}
}

/// A `View` widget that can safely run an asynchronous task tied to its lifecycle.
///
/// - If the widget gets dropped, the running task will be aborted automatically.
/// - Running a new task will abort the previous one, if it exists.
///
/// Also, the running task can access the `UiRunner` of the widget, allowing it to
/// defer UI updates from the task.
#[derive(Live, Widget, LiveHook)]
pub struct AsyncView {
    #[deref]
    deref: View,

    #[rust]
    abort_handle: Option<AbortHandle>,
}

impl Widget for AsyncView {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope)
    }
}
impl AsyncView {
    /// Runs the provided future, with access to this widget through an `UiRunner`,
    /// and cancelling any previously running task.
    ///
    /// The task will be aborted automatically if the widget is dropped.
    pub fn spawn<F, Fut>(&mut self, future_cb: F)
    where
        F: FnOnce(UiRunner<Self>) -> Fut + 'static,
        Fut: PlatformSendFuture + 'static,
    {
        self.abort();

        let future = future_cb(self.ui_runner());
        let (future, abort_handle) = futures::future::abortable(future);

        self.abort_handle = Some(abort_handle);
        spawn(async move {
            let _ = future.await;
        });
    }

    /// Manually aborts the running task, if it exists.
    pub fn abort(&mut self) {
        if let Some(abort_handle) = self.abort_handle.take() {
            abort_handle.abort();
        }
    }
}

impl Drop for AsyncView {
    fn drop(&mut self) {
        self.abort();
    }
}
