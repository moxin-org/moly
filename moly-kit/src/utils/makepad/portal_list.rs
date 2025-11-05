//! Utilities to deal with [PortalList] widget.

use makepad_widgets::*;
use std::ops::RangeBounds;

/// Iterator over a subset of portal list items.
pub struct ItemsRangeIter<R: RangeBounds<usize>> {
    list: PortalListRef,
    range: R,
    current: usize,
}

impl<R: RangeBounds<usize>> ItemsRangeIter<R> {
    pub fn new(list: PortalListRef, range: R) -> Self {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&start) => start,
            std::ops::Bound::Excluded(&start) => start + 1,
            std::ops::Bound::Unbounded => 0,
        };

        Self {
            list,
            range,
            current: start,
        }
    }
}

impl<R: RangeBounds<usize>> Iterator for ItemsRangeIter<R> {
    type Item = (usize, WidgetRef);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.range.contains(&self.current) {
            return None;
        }

        // Currently PortalList doesn't expose its children in an unconditional way,
        // that why I'm creating this iterator on the first place, on top of `get_item`
        // which esentialy does a hash map lookup.
        let Some((_, item)) = self.list.get_item(self.current) else {
            return None;
        };

        self.current += 1;
        Some((self.current - 1, item))
    }
}
