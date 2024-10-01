use makepad_widgets::*;
use std::sync::{Arc, Mutex};

/// Unlocks the potential of modifying the UI "directly" from a different thread.
///
/// Closoures passed to `Dragonfly::run` will be executed on the UI thread with
/// access to your widget and context.
///
/// The name "Dragonfly" is inspired by the insect's ability to move in all directions.
/// (ctually, it's just the first word it came to my mind, I accept suggestions.
#[derive(Debug, Clone)]
pub struct Dragonfly {
    id: usize,
}

impl Dragonfly {
    /// Crate a new instance of `Dragonfly`.
    /// Only this instance and its clones can handle the functions it schedules.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        Self { id }
    }

    /// Handle the event and execute the scheduled functions.
    /// Call this in your `handle_event` method.
    ///
    /// Scheduled functons can only run once, so after being run here, they will
    /// not run anywhere else. Therefore, you should call this method from a single
    /// place in your code.
    ///
    /// If you stored this instance in the target itself (for example, your widget),
    /// you will need to call this like `self.dragonfly.clone().handle(self, cx, event)`.
    pub fn handle<T: 'static>(&self, target: &mut T, cx: &mut Cx, event: &Event) {
        if let Event::Actions(actions) = event {
            for action in actions {
                if let Some(action) = action.downcast_ref::<DragonflyAction<T>>() {
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

    /// Run the provided closure on the UI thread.
    pub fn run<T: 'static>(&self, f: impl FnOnce(&mut T, &mut Cx) + Send + 'static) {
        let action = DragonflyAction {
            f: Arc::new(Mutex::new(Some(Box::new(f)))),
            id: self.id,
        };

        Cx::post_action(action);
    }

    /// Spawn a new thread and run the provided closure.
    ///
    /// This is the same as cloning the `Dragonfly` instance and calling
    /// `std::thread::spawn` manually.
    pub fn spawn(&self, f: impl FnOnce(Dragonfly) + Send + 'static) {
        let dragonfly = self.clone();
        std::thread::spawn(move || {
            f(dragonfly);
        });
    }
}

struct DragonflyAction<T> {
    f: Arc<Mutex<Option<Box<dyn FnOnce(&mut T, &mut Cx) + Send + 'static>>>>,
    id: usize,
}

impl<T> std::fmt::Debug for DragonflyAction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mutator")
            .field("id", &self.id)
            .field("f", &"{irrelevant}")
            .finish()
    }
}
