use makepad_widgets::{Action, Actions, Event};
use std::sync::Once;

static mut EMPTY: *const Vec<Action> = std::ptr::null();
static INIT: Once = Once::new();

pub trait EventExt {
    fn actions(&self) -> &Actions;
}

impl EventExt for Event {
    /// Extract the actions from this events (if any).
    ///
    /// A workaround is used when the Makepad's Event is not an Actions variant
    /// to return a 'static immutable reference to a global empty vector.
    fn actions(&self) -> &Actions {
        match self {
            Event::Actions(actions) => actions,
            _ => unsafe {
                // This is safe just because it's an immutable reference and the
                // vector is empty.
                //
                // A non-unsafe implementation would require a thread_local storage
                // since the Action type is not Send but that may be even dangerous
                // as it would need to leak a vector for each thread (which should only be one
                // but who knows). So I preferred goging unsafe here.

                INIT.call_once(|| {
                    // Leak an empty vector to get 'static reference
                    let empty = Box::new(Vec::new());
                    EMPTY = Box::leak(empty);
                });
                &*EMPTY // Convert raw pointer to reference
            },
        }
    }
}
