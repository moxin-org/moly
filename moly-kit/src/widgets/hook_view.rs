use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub HookView = {{HookView}} {}
}

#[derive(Live, Widget, LiveHook)]
pub struct HookView {
    #[deref]
    pub deref: View,

    #[rust]
    on_before_event: Option<Box<dyn FnMut(&mut Self, &mut Cx, &Event, &mut Scope)>>,

    #[rust]
    on_after_event: Option<Box<dyn FnMut(&mut Self, &mut Cx, &Event, &mut Scope)>>,
}

impl Widget for HookView {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if let Some(mut on_before_event) = self.on_before_event.take() {
            on_before_event(self, cx, event, scope);
            self.on_before_event = Some(on_before_event);
        }

        self.deref.handle_event(cx, event, scope);

        if let Some(mut on_after_event) = self.on_after_event.take() {
            on_after_event(self, cx, event, scope);
            self.on_after_event = Some(on_after_event);
        }
    }
}

impl HookView {
    pub fn on_before_event(
        &mut self,
        f: impl FnMut(&mut Self, &mut Cx, &Event, &mut Scope) + 'static,
    ) -> &mut Self {
        self.on_before_event = Some(Box::new(f));
        self
    }

    pub fn on_after_event(
        &mut self,
        f: impl FnMut(&mut Self, &mut Cx, &Event, &mut Scope) + 'static,
    ) -> &mut Self {
        self.on_after_event = Some(Box::new(f));
        self
    }
}

impl HookViewRef {
    /// Immutable access to the underlying [`HookView`].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read(&self) -> std::cell::Ref<'_, HookView> {
        self.borrow().unwrap()
    }

    /// Mutable access to the underlying [`HookView`].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write(&mut self) -> std::cell::RefMut<'_, HookView> {
        self.borrow_mut().unwrap()
    }
}
