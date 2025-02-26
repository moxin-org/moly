use makepad_widgets::{Action, Event};

pub trait EventExt {
    /// Returns `&[Action]` (either the event's actions or an empty fallback).
    fn actions(&self) -> &[Action];
}

impl EventExt for Event {
    fn actions(&self) -> &[Action] {
        match self {
            Event::Actions(actions) => actions.as_slice(),
            _ => &[],
        }
    }
}
