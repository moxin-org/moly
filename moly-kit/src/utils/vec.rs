/// Set of indexes, unique and sorted.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct IndexSet {
    indices: Vec<usize>,
}

impl std::ops::Deref for IndexSet {
    type Target = [usize];

    fn deref(&self) -> &Self::Target {
        &self.indices
    }
}

impl From<Vec<usize>> for IndexSet {
    fn from(mut indices: Vec<usize>) -> Self {
        indices.sort_unstable();
        indices.dedup();
        Self { indices }
    }
}

/// Ergonomic, debuggable and loggable mutation to a `Vec<T>`.
///
/// Variants in this enum may represent overlapping operations as they are designed
/// for ergonomics and clarity. To analyze the precise changes to a vector, use the
/// [`VecMutation::log`] method which produces a detailed log of the changes that
/// a mutation will apply.
#[derive(Debug, Clone, PartialEq)]
pub enum VecMutation<T: Clone> {
    /// Replaces the given range with new elements.
    ///
    /// Splice is capable of both inserting and removing elements.
    ///
    /// When analyzed as a [`VecEffect`], this mutation is equivalent to a `Remove` followed by an `Insert`.
    Splice(usize, usize, Vec<T>),
    /// Inserts many elements at the given index.
    ///
    /// Same as `Splice` with an empty removal range.
    InsertMany(usize, Vec<T>),
    /// Inserts one element at the given index.
    ///
    /// Same as [`InsertMany`] but for a single element.
    InsertOne(usize, T),
    /// Appends many elements to the end of the vec.
    ///
    /// Similar to `InsertMany` but targeting the end of the vec when applied.
    Extend(Vec<T>),
    /// Appends one element to the end of the vec.
    ///
    /// Same as `Extend` but for a single element.
    Push(T),
    /// Removes a range of elements.
    ///
    /// Same as `Splice` with an empty insertion list.
    RemoveRange(usize, usize),
    /// Removes a single element at the given index.
    ///
    /// Same as `RemoveRange` but for a single element.
    RemoveOne(usize),
    /// Remove many sparse elements at once.
    ///
    /// When analyzed as a [`VecEffect`], this mutation is equivalent to multiple sparse `Remove`s.
    ///
    /// Note: This is more efficient than multiple `RemoveOne` mutations, as all
    /// removals are performed in a single `retain` pass. However, building the
    /// index list itself can be expensive anyways, so use this with caution.
    RemoveMany(IndexSet),
    /// Removes the last element from the vec.
    ///
    /// When analyzed as a [`VecEffect`], this mutation is equivalent to a `Remove`.
    RemoveLast,
    /// Removes all elements from the vec.
    ///
    /// Same as `RemoveRange` targeting the full range of the vec.
    Clear,
    /// Updates the element at the given index.
    ///
    /// When analyzed as a [`VecEffect`], this mutation is equivalent to an `Update`.
    Update(usize, T),
    /// Updates the last element in the vec.
    ///
    /// When analyzed as a [`VecEffect`], this mutation is equivalent to an `Update`.
    UpdateLast(T),
    /// Replaces the entire contents of the vec with the given elements.
    ///
    /// When analyzed as a [`VecEffect`], this mutation is equivalent to a `Remove`
    /// of the full previous contents followed by an `Insert` of the new contents.
    Set(Vec<T>),
}

impl<T: Clone> VecMutation<T> {
    /// Apply the changes represented by this mutation to the given vec.
    pub fn apply(self, list: &mut Vec<T>) {
        match self {
            Self::Splice(start, end, items) => {
                list.splice(start..end, items);
            }
            Self::InsertMany(index, items) => {
                list.splice(index..index, items);
            }
            Self::InsertOne(index, item) => {
                list.insert(index, item);
            }
            Self::Extend(items) => {
                list.extend(items);
            }
            Self::Push(item) => {
                list.push(item);
            }
            Self::RemoveRange(start, end) => {
                list.drain(start..end);
            }
            Self::RemoveOne(index) => {
                list.remove(index);
            }
            Self::RemoveMany(indices) => {
                let mut iter = indices.into_iter();
                let mut next_to_remove = iter.next().copied();
                let mut current = 0;
                list.retain_mut(|_| {
                    let to_retain = match next_to_remove {
                        Some(idx) if idx == current => {
                            next_to_remove = iter.next().copied();
                            false
                        }
                        _ => true,
                    };

                    current += 1;
                    to_retain
                });
            }
            Self::Clear => {
                list.clear();
            }
            Self::RemoveLast => {
                list.pop();
            }
            Self::Update(index, item) => {
                list[index] = item;
            }
            Self::UpdateLast(item) => {
                *list.last_mut().unwrap() = item;
            }
            Self::Set(items) => {
                *list = items;
            }
        }
    }

    /// Use a function to craft a [`VecMutation::Update`] using mutable semantics.
    ///
    /// This clones the target item on construction.
    pub fn update_with(target: &[T], index: usize, updater: impl FnOnce(&mut T)) -> VecMutation<T> {
        let mut item = target[index].clone();
        updater(&mut item);
        VecMutation::Update(index, item)
    }

    /// Use a function to craft a [`VecMutation::UpdateLast`] using mutable semantics.
    ///
    /// This clones the target item on construction.
    pub fn update_last_with(target: &[T], updater: impl FnOnce(&mut T)) -> VecMutation<T> {
        let mut item = target.last().unwrap().clone();
        updater(&mut item);
        VecMutation::UpdateLast(item)
    }

    /// Constructs a [`VecMutation::RemoveMany`] using filtering semantics.
    ///
    /// Warning: This will allocate a `Vec` to hold all the matched indices.
    /// Read the documentation of [`VecMutation::RemoveMany`] for more details.
    pub fn remove_many_from_filter(
        target: &[T],
        mut filter: impl FnMut(usize, &T) -> bool,
    ) -> VecMutation<T> {
        let indices: Vec<usize> = target
            .iter()
            .enumerate()
            .filter_map(|(i, item)| if filter(i, item) { Some(i) } else { None })
            .collect();
        VecMutation::RemoveMany(IndexSet::from(indices))
    }

    pub fn effect<'a>(
        &'a self,
        target: &'a [T],
    ) -> Box<dyn Iterator<Item = VecEffect<'a, T>> + 'a> {
        match self {
            Self::Splice(start, end, items) => Box::new(
                std::iter::once(VecEffect::Remove(*start, *end, &target[*start..*end]))
                    .chain(std::iter::once(VecEffect::Insert(*start, items))),
            ),
            Self::InsertMany(index, items) => {
                Box::new(std::iter::once(VecEffect::Insert(*index, items)))
            }
            Self::InsertOne(index, item) => Box::new(std::iter::once(VecEffect::Insert(
                *index,
                std::slice::from_ref(item),
            ))),
            Self::Extend(items) => {
                Box::new(std::iter::once(VecEffect::Insert(target.len(), items)))
            }
            Self::Push(item) => Box::new(std::iter::once(VecEffect::Insert(
                target.len(),
                std::slice::from_ref(item),
            ))),
            Self::RemoveRange(start, end) => Box::new(std::iter::once(VecEffect::Remove(
                *start,
                *end,
                &target[*start..*end],
            ))),
            Self::RemoveOne(index) => Box::new(std::iter::once(VecEffect::Remove(
                *index,
                index + 1,
                &target[*index..*index + 1],
            ))),
            Self::RemoveMany(indices) => {
                Box::new(indices.iter().map(move |&index| {
                    VecEffect::Remove(index, index + 1, &target[index..index + 1])
                }))
            }
            Self::Clear => Box::new(std::iter::once(VecEffect::Remove(0, target.len(), target))),
            Self::RemoveLast => Box::new(std::iter::once(VecEffect::Remove(
                target.len() - 1,
                target.len(),
                &target[target.len() - 1..],
            ))),
            Self::Update(index, item) => Box::new(std::iter::once(VecEffect::Update(
                *index,
                &target[*index],
                item,
            ))),
            Self::UpdateLast(item) => Box::new(std::iter::once(VecEffect::Update(
                target.len() - 1,
                &target[target.len() - 1],
                item,
            ))),
            Self::Set(items) => Box::new(
                std::iter::once(VecEffect::Remove(0, target.len(), target))
                    .chain(std::iter::once(VecEffect::Insert(0, items))),
            ),
        }
    }
}

/// A primitive operation that will be performed on a `Vec<T>` as a result of a
/// higher level and more optimized [`VecMutation`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VecEffect<'a, T> {
    /// The given NEW items will be inserted at the given index.
    ///
    /// Means the length of the list will INCREASE by the length of `items`.
    Insert(usize, &'a [T]),
    /// The EXISTING item at the given index will be replaced with a new value.
    ///
    /// First sample is the previous value, second sample is the new value.
    ///
    /// Means the length of the list will STAY THE SAME.
    // WARNING: Never emit an Update for an index that does not exist!
    Update(usize, &'a T, &'a T),
    /// The specified range of EXISTING items will be removed.
    ///
    /// The sample at the end is the slice of items that will be removed.
    ///
    /// Means the length of the list will DECREASE by `end - start`.
    Remove(usize, usize, &'a [T]),
}
