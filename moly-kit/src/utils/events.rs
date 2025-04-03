use makepad_widgets::{Action, Event, WidgetAction, WidgetActionCast};

pub trait EventExt {
    /// Returns `&[Action]` (either the event's actions or an empty fallback).
    fn actions(&self) -> &[Action];

    /// Filtered iterator over widget actions.
    fn widget_actions(&self) -> impl Iterator<Item = &WidgetAction>;
}

impl EventExt for Event {
    fn actions(&self) -> &[Action] {
        match self {
            Event::Actions(actions) => actions.as_slice(),
            _ => &[],
        }
    }

    fn widget_actions(&self) -> impl Iterator<Item = &WidgetAction> {
        self.actions()
            .iter()
            .filter_map(|action| action.as_widget_action())
    }
}
