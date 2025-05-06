use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub Slot = {{Slot}} {}
}

/// A wrapper widget whose content can be replaced from Rust.
#[derive(Live, Widget)]
pub struct Slot {
    #[wrap]
    wrap: WidgetRef,

    /// The content defined in the DSL to be shown if it hasn't been overridden.
    ///
    /// If overridden, this can still be restored using [Self::restore].
    #[live]
    default: WidgetRef,
}

impl Widget for Slot {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.wrap.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.wrap.handle_event(cx, event, scope)
    }
}

impl LiveHook for Slot {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        self.wrap = self.default.clone();
    }
}

impl Slot {
    /// Replace the current widget with a new one.
    pub fn replace(&mut self, widget: WidgetRef) {
        self.wrap = widget;
    }

    /// Restore the default/original widget.
    ///
    /// Same as `self.replace(self.default())`.
    pub fn restore(&mut self) {
        self.wrap = self.default.clone();
    }

    /// Get the current widget.
    pub fn current(&self) -> WidgetRef {
        self.wrap.clone()
    }

    /// Get the default/original widget.
    pub fn default(&self) -> WidgetRef {
        self.default.clone()
    }
}

impl SlotRef {
    /// See [Slot::replace].
    pub fn replace(&mut self, widget: WidgetRef) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };

        inner.replace(widget);
    }

    /// See [Slot::restore].
    pub fn restore(&mut self) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };

        inner.restore();
    }

    /// See [Slot::current].
    #[allow(dead_code)]
    pub fn current(&self) -> WidgetRef {
        let Some(inner) = self.borrow() else {
            return WidgetRef::empty();
        };

        inner.current()
    }

    /// See [Slot::default].
    pub fn default(&self) -> WidgetRef {
        let Some(inner) = self.borrow() else {
            return WidgetRef::empty();
        };

        inner.default()
    }
}
