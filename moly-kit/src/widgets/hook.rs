//! A widget that allows hooking into Makepad's lifecycle with closures.
//!
//! Use [`crate::widgets::HookView`] instead if you need [`View`]'s behavior.

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub Hook = {{Hook}} {}
}

#[derive(Live, Widget, LiveHook)]
pub struct Hook {
    #[wrap]
    #[live]
    pub wrap: WidgetRef,

    #[rust]
    on_before_event: Option<Box<dyn FnMut(&mut Self, &mut Cx, &Event, &mut Scope)>>,

    #[rust]
    on_after_event: Option<Box<dyn FnMut(&mut Self, &mut Cx, &Event, &mut Scope)>>,
}

impl Widget for Hook {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.wrap.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if let Some(mut on_before_event) = self.on_before_event.take() {
            on_before_event(self, cx, event, scope);
            self.on_before_event = Some(on_before_event);
        }

        self.wrap.handle_event(cx, event, scope);

        if let Some(mut on_after_event) = self.on_after_event.take() {
            on_after_event(self, cx, event, scope);
            self.on_after_event = Some(on_after_event);
        }
    }
}

impl Hook {
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

impl HookRef {
    /// Immutable access to the underlying [`Hook`].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read(&self) -> std::cell::Ref<'_, Hook> {
        self.borrow().unwrap()
    }

    /// Mutable access to the underlying [`Hook`].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write(&mut self) -> std::cell::RefMut<'_, Hook> {
        self.borrow_mut().unwrap()
    }
}
