use makepad_widgets::*;

live_design!(
    List = {{List}} {
        flow: Down,
        width: Fill,
        height: Fill,
    }
);

/// Minimalistic list of dynamic widgets created from your data, eagerly rendered and
/// with a known size.
#[derive(Live, Widget, LiveHook)]
pub struct List {
    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[redraw]
    #[rust]
    area: Area,

    #[rust]
    items: Vec<WidgetRef>,
}

impl Widget for List {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.items.iter().for_each(|item| {
            item.handle_event(cx, event, scope);
        });
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        self.items.iter().for_each(|item| {
            item.draw_all(cx, scope);
        });
        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}

impl List {
    /// Set to an already generated list of widgets.
    pub fn set_items(&mut self, items: Vec<WidgetRef>) {
        self.items = items;
    }

    /// Build each widget mapping them from your data.
    pub fn compute_from<T, I: Iterator<Item = T>>(
        &mut self,
        iter: I,
        f: impl FnMut(T) -> WidgetRef,
    ) {
        self.set_items(iter.map(f).collect());
    }

    /// Returns the number of items in the list.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns an iterator over the widgets in the list.
    pub fn items(&self) -> impl Iterator<Item = &WidgetRef> {
        self.items.iter()
    }
}

impl ListRef {
    /// Calls `set_items` on the inner widget.
    pub fn set_items(&mut self, items: Vec<WidgetRef>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(items);
        }
    }

    /// Calls `compute_from` on the inner widget.
    pub fn compute_from<T, I: Iterator<Item = T>>(&self, iter: I, f: impl FnMut(T) -> WidgetRef) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.compute_from(iter, f);
        }
    }

    /// Calls `len` on the inner widget.
    pub fn len(&self) -> usize {
        self.borrow().map_or(0, |inner| inner.len())
    }
}
