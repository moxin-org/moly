use makepad_widgets::*;
use std::sync::{Arc, Mutex};

/// Run code on the UI thread from another thread.
///
/// Allows you to mix non-blocking threaded code, with code that reads and updates
/// your widget in the UI thread.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiRunner {
    /// Trick to later distinguish actions sent globally thru `Cx::post_action`.
    id: usize,
}

impl UiRunner {
    /// Create a new isolated instance.
    ///
    /// Functions scheduled thru this instance will not interfere with functions
    /// scheduled thru other instances.
    ///
    /// The instance itself can be copied and passed around.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        Self { id }
    }

    /// Handle all functions scheduled thru this instance.
    ///
    /// You should call this once from yout `handle_event` method, like:
    ///
    /// ```rust
    /// fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
    ///    // ... handle other stuff ...
    ///    self.ui_runner.handle(cx, event, self);
    /// }
    /// ```
    ///
    /// Once a function has been handled, it will never run again.
    pub fn handle<T: 'static>(self, cx: &mut Cx, event: &Event, target: &mut T) {
        if let Event::Actions(actions) = event {
            for action in actions {
                if let Some(action) = action.downcast_ref::<UiRunnerAction<T>>() {
                    if action.id != self.id {
                        continue;
                    }

                    if let Some(f) = action.f.lock().unwrap().take() {
                        (f)(target, cx);
                    }
                }
            }
        }
    }

    /// Schedule the provided closure to run on the UI thread.
    ///
    /// Note: You will need to specify the type of the target widget, and it should
    /// match the target type being handled by the `UiRunner::handle` method, or it
    /// will be ignored.
    pub fn run<T: 'static>(self, f: impl FnOnce(&mut T, &mut Cx) + Send + 'static) {
        let action = UiRunnerAction {
            f: Arc::new(Mutex::new(Some(Box::new(f)))),
            id: self.id,
        };

        Cx::post_action(action);
    }
}

struct UiRunnerAction<T> {
    f: Arc<Mutex<Option<Box<dyn FnOnce(&mut T, &mut Cx) + Send + 'static>>>>,
    id: usize,
}

impl<T> std::fmt::Debug for UiRunnerAction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiRunnerAction")
            .field("id", &self.id)
            .field("f", &"...")
            .finish()
    }
}
