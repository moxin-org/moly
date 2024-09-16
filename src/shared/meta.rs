use std::{
    any::Any,
    cell::{Ref, RefMut},
};

use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    Meta = {{Meta}} { width: 0, height: 0 }
}

/// A non visual widget that can store any value.
///
/// Useful paired with `PortalList` or `ComputedList` to link data to dynamic widgets.
/// Safer than manually keeping a mapping between data and widgets since the data will be
/// destroyed automatically if this widget is destroyed. No risk of forgetting a call to `clear`.
#[derive(Live, LiveHook, Widget)]
pub struct Meta {
    #[walk]
    walk: Walk,

    #[redraw]
    area: Area,

    #[rust]
    value: Option<Box<dyn Any>>,
}

impl Widget for Meta {
    fn draw_walk(&mut self, _cx: &mut Cx2d, _scope: &mut Scope, _walk: Walk) -> DrawStep {
        DrawStep::done()
    }

    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}
}

impl Meta {
    /// Sets the value that this widget will store.
    ///
    /// Overrides any previous value.
    pub fn set_value<T: Any>(&mut self, value: T) {
        self.value = Some(Box::new(value));
    }

    /// Gets the value stored in this widget immutably.
    ///
    /// Returns `None` if there is no value or if the value is not of the requested type.
    pub fn get_value<T: Any>(&self) -> Option<&T> {
        self.value.as_ref().and_then(|v| v.downcast_ref())
    }

    /// Gets the value stored in this widget mutably.
    ///
    /// Returns `None` if there is no value or if the value is not of the requested type.
    pub fn get_value_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.value.as_mut().and_then(|v| v.downcast_mut())
    }
}

impl MetaRef {
    /// Calls `get_value` on the inner widget.
    pub fn get_value<T: Any>(&self) -> Option<Ref<T>> {
        let Some(inner) = self.borrow() else {
            return None;
        };

        if inner.get_value::<T>().is_none() {
            return None;
        }

        Some(Ref::map(inner, |inner| inner.get_value::<T>().unwrap()))
    }

    /// Calls `get_value_mut` on the inner widget.
    pub fn get_value_mut<T: Any>(&self) -> Option<RefMut<T>> {
        let Some(inner) = self.borrow_mut() else {
            return None;
        };

        if inner.get_value::<T>().is_none() {
            return None;
        }

        Some(RefMut::map(inner, |inner| {
            inner.get_value_mut::<T>().unwrap()
        }))
    }

    /// Calls `set_value` on the inner widget.
    pub fn set_value<T: Any>(&mut self, value: T) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };

        inner.set_value(value);
    }
}
