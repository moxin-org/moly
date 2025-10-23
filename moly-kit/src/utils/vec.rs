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

    /// Constructs a [`VecMutation::RemoveMany`] using retention semantics.
    ///
    /// The `retain` predicate should return `true` for elements to **keep** and `false` for elements to **remove**.
    /// This matches the semantics of Rust's standard `Vec::retain` method.
    ///
    /// Warning: This will allocate a `Vec` to hold all the matched indices.
    /// Read the documentation of [`VecMutation::RemoveMany`] for more details.
    pub fn remove_many_with_retain(
        target: &[T],
        mut retain: impl FnMut(usize, &T) -> bool,
    ) -> VecMutation<T> {
        let indices: Vec<usize> = target
            .iter()
            .enumerate()
            .filter_map(|(i, item)| if !retain(i, item) { Some(i) } else { None })
            .collect();
        VecMutation::RemoveMany(IndexSet::from(indices))
    }

    pub fn effects<'a>(
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

#[cfg(test)]
mod tests {
    use super::{IndexSet, VecMutation};

    #[test]
    fn test_splice_replace() {
        let mut vec = vec![1, 2, 3, 4, 5];
        VecMutation::Splice(1, 3, vec![10, 20]).apply(&mut vec);
        assert_eq!(vec, vec![1, 10, 20, 4, 5]);
    }

    #[test]
    fn test_splice_insert_only() {
        let mut vec = vec![1, 2, 3];
        VecMutation::Splice(1, 1, vec![10, 20]).apply(&mut vec);
        assert_eq!(vec, vec![1, 10, 20, 2, 3]);
    }

    #[test]
    fn test_splice_remove_only() {
        let mut vec = vec![1, 2, 3, 4, 5];
        VecMutation::Splice(1, 4, vec![]).apply(&mut vec);
        assert_eq!(vec, vec![1, 5]);
    }

    #[test]
    fn test_insert_many() {
        let mut vec = vec![1, 2, 3];
        VecMutation::InsertMany(1, vec![10, 20, 30]).apply(&mut vec);
        assert_eq!(vec, vec![1, 10, 20, 30, 2, 3]);
    }

    #[test]
    fn test_insert_many_at_start() {
        let mut vec = vec![1, 2, 3];
        VecMutation::InsertMany(0, vec![10, 20]).apply(&mut vec);
        assert_eq!(vec, vec![10, 20, 1, 2, 3]);
    }

    #[test]
    fn test_insert_many_at_end() {
        let mut vec = vec![1, 2, 3];
        VecMutation::InsertMany(3, vec![10, 20]).apply(&mut vec);
        assert_eq!(vec, vec![1, 2, 3, 10, 20]);
    }

    #[test]
    fn test_insert_one() {
        let mut vec = vec![1, 2, 3];
        VecMutation::InsertOne(1, 10).apply(&mut vec);
        assert_eq!(vec, vec![1, 10, 2, 3]);
    }

    #[test]
    fn test_insert_one_at_start() {
        let mut vec = vec![1, 2, 3];
        VecMutation::InsertOne(0, 10).apply(&mut vec);
        assert_eq!(vec, vec![10, 1, 2, 3]);
    }

    #[test]
    fn test_insert_one_at_end() {
        let mut vec = vec![1, 2, 3];
        VecMutation::InsertOne(3, 10).apply(&mut vec);
        assert_eq!(vec, vec![1, 2, 3, 10]);
    }

    #[test]
    fn test_extend() {
        let mut vec = vec![1, 2, 3];
        VecMutation::Extend(vec![10, 20, 30]).apply(&mut vec);
        assert_eq!(vec, vec![1, 2, 3, 10, 20, 30]);
    }

    #[test]
    fn test_extend_empty_vec() {
        let mut vec: Vec<i32> = vec![];
        VecMutation::Extend(vec![10, 20]).apply(&mut vec);
        assert_eq!(vec, vec![10, 20]);
    }

    #[test]
    fn test_push() {
        let mut vec = vec![1, 2, 3];
        VecMutation::Push(10).apply(&mut vec);
        assert_eq!(vec, vec![1, 2, 3, 10]);
    }

    #[test]
    fn test_push_to_empty() {
        let mut vec: Vec<i32> = vec![];
        VecMutation::Push(10).apply(&mut vec);
        assert_eq!(vec, vec![10]);
    }

    #[test]
    fn test_remove_range() {
        let mut vec = vec![1, 2, 3, 4, 5];
        VecMutation::RemoveRange(1, 4).apply(&mut vec);
        assert_eq!(vec, vec![1, 5]);
    }

    #[test]
    fn test_remove_range_from_start() {
        let mut vec = vec![1, 2, 3, 4, 5];
        VecMutation::RemoveRange(0, 2).apply(&mut vec);
        assert_eq!(vec, vec![3, 4, 5]);
    }

    #[test]
    fn test_remove_range_to_end() {
        let mut vec = vec![1, 2, 3, 4, 5];
        VecMutation::RemoveRange(2, 5).apply(&mut vec);
        assert_eq!(vec, vec![1, 2]);
    }

    #[test]
    fn test_remove_one() {
        let mut vec = vec![1, 2, 3, 4, 5];
        VecMutation::RemoveOne(2).apply(&mut vec);
        assert_eq!(vec, vec![1, 2, 4, 5]);
    }

    #[test]
    fn test_remove_one_first() {
        let mut vec = vec![1, 2, 3];
        VecMutation::RemoveOne(0).apply(&mut vec);
        assert_eq!(vec, vec![2, 3]);
    }

    #[test]
    fn test_remove_one_last() {
        let mut vec = vec![1, 2, 3];
        VecMutation::RemoveOne(2).apply(&mut vec);
        assert_eq!(vec, vec![1, 2]);
    }

    #[test]
    fn test_remove_many_sparse() {
        let mut vec = vec![1, 2, 3, 4, 5, 6, 7];
        let indices = IndexSet::from(vec![1, 3, 5]);
        VecMutation::RemoveMany(indices).apply(&mut vec);
        assert_eq!(vec, vec![1, 3, 5, 7]);
    }

    #[test]
    fn test_remove_many_single() {
        let mut vec = vec![1, 2, 3, 4, 5];
        let indices = IndexSet::from(vec![2]);
        VecMutation::RemoveMany(indices).apply(&mut vec);
        assert_eq!(vec, vec![1, 2, 4, 5]);
    }

    #[test]
    fn test_remove_many_consecutive() {
        let mut vec = vec![1, 2, 3, 4, 5];
        let indices = IndexSet::from(vec![1, 2, 3]);
        VecMutation::RemoveMany(indices).apply(&mut vec);
        assert_eq!(vec, vec![1, 5]);
    }

    #[test]
    fn test_remove_many_all() {
        let mut vec = vec![1, 2, 3];
        let indices = IndexSet::from(vec![0, 1, 2]);
        VecMutation::RemoveMany(indices).apply(&mut vec);
        assert_eq!(vec, Vec::<i32>::new());
    }

    #[test]
    fn test_remove_many_empty() {
        let mut vec = vec![1, 2, 3];
        let indices = IndexSet::from(vec![]);
        VecMutation::RemoveMany(indices).apply(&mut vec);
        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_remove_many_duplicates_in_input() {
        // IndexSet should deduplicate
        let mut vec = vec![1, 2, 3, 4, 5];
        let indices = IndexSet::from(vec![1, 1, 3, 3, 1]);
        VecMutation::RemoveMany(indices).apply(&mut vec);
        assert_eq!(vec, vec![1, 3, 5]);
    }

    #[test]
    fn test_remove_many_unsorted_input() {
        // IndexSet should sort
        let mut vec = vec![1, 2, 3, 4, 5];
        let indices = IndexSet::from(vec![3, 1, 4]);
        VecMutation::RemoveMany(indices).apply(&mut vec);
        assert_eq!(vec, vec![1, 3]);
    }

    #[test]
    fn test_clear() {
        let mut vec = vec![1, 2, 3, 4, 5];
        VecMutation::Clear.apply(&mut vec);
        assert_eq!(vec, Vec::<i32>::new());
    }

    #[test]
    fn test_clear_empty() {
        let mut vec: Vec<i32> = vec![];
        VecMutation::Clear.apply(&mut vec);
        assert_eq!(vec, Vec::<i32>::new());
    }

    #[test]
    fn test_remove_last() {
        let mut vec = vec![1, 2, 3];
        VecMutation::RemoveLast.apply(&mut vec);
        assert_eq!(vec, vec![1, 2]);
    }

    #[test]
    fn test_remove_last_single_element() {
        let mut vec = vec![1];
        VecMutation::RemoveLast.apply(&mut vec);
        assert_eq!(vec, Vec::<i32>::new());
    }

    #[test]
    fn test_update() {
        let mut vec = vec![1, 2, 3, 4, 5];
        VecMutation::Update(2, 100).apply(&mut vec);
        assert_eq!(vec, vec![1, 2, 100, 4, 5]);
    }

    #[test]
    fn test_update_first() {
        let mut vec = vec![1, 2, 3];
        VecMutation::Update(0, 100).apply(&mut vec);
        assert_eq!(vec, vec![100, 2, 3]);
    }

    #[test]
    fn test_update_last() {
        let mut vec = vec![1, 2, 3];
        VecMutation::UpdateLast(100).apply(&mut vec);
        assert_eq!(vec, vec![1, 2, 100]);
    }

    #[test]
    fn test_update_last_single_element() {
        let mut vec = vec![1];
        VecMutation::UpdateLast(100).apply(&mut vec);
        assert_eq!(vec, vec![100]);
    }

    #[test]
    fn test_set() {
        let mut vec = vec![1, 2, 3];
        VecMutation::Set(vec![10, 20, 30, 40]).apply(&mut vec);
        assert_eq!(vec, vec![10, 20, 30, 40]);
    }

    #[test]
    fn test_set_empty() {
        let mut vec = vec![1, 2, 3];
        VecMutation::Set(vec![]).apply(&mut vec);
        assert_eq!(vec, Vec::<i32>::new());
    }

    #[test]
    fn test_set_to_empty() {
        let mut vec: Vec<i32> = vec![];
        VecMutation::Set(vec![10, 20]).apply(&mut vec);
        assert_eq!(vec, vec![10, 20]);
    }

    #[test]
    fn test_update_with() {
        let vec = vec![1, 2, 3, 4, 5];
        let mut vec_mut = vec.clone();
        let mutation = VecMutation::update_with(&vec, 2, |x| *x *= 10);
        mutation.apply(&mut vec_mut);
        assert_eq!(vec_mut, vec![1, 2, 30, 4, 5]);
    }

    #[test]
    fn test_update_with_complex_modification() {
        #[derive(Debug, Clone, PartialEq)]
        struct Item {
            id: i32,
            value: String,
        }

        let vec = vec![
            Item {
                id: 1,
                value: "a".to_string(),
            },
            Item {
                id: 2,
                value: "b".to_string(),
            },
            Item {
                id: 3,
                value: "c".to_string(),
            },
        ];
        let mut vec_mut = vec.clone();
        let mutation = VecMutation::update_with(&vec, 1, |item| {
            item.id = 20;
            item.value = "modified".to_string();
        });
        mutation.apply(&mut vec_mut);
        assert_eq!(
            vec_mut,
            vec![
                Item {
                    id: 1,
                    value: "a".to_string()
                },
                Item {
                    id: 20,
                    value: "modified".to_string()
                },
                Item {
                    id: 3,
                    value: "c".to_string()
                },
            ]
        );
    }

    #[test]
    fn test_update_last_with() {
        let vec = vec![1, 2, 3, 4, 5];
        let mut vec_mut = vec.clone();
        let mutation = VecMutation::update_last_with(&vec, |x| *x *= 10);
        mutation.apply(&mut vec_mut);
        assert_eq!(vec_mut, vec![1, 2, 3, 4, 50]);
    }

    #[test]
    fn test_remove_many_with_retain_basic() {
        let vec = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut vec_mut = vec.clone();
        // Keep odd numbers (remove even numbers)
        let mutation = VecMutation::remove_many_with_retain(&vec, |_, &item| item % 2 != 0);
        mutation.apply(&mut vec_mut);
        assert_eq!(vec_mut, vec![1, 3, 5, 7]);
    }

    #[test]
    fn test_remove_many_with_retain_none_removed() {
        let vec = vec![1, 3, 5, 7];
        let mut vec_mut = vec.clone();
        // Keep all odd numbers (all elements match)
        let mutation = VecMutation::remove_many_with_retain(&vec, |_, &item| item % 2 != 0);
        mutation.apply(&mut vec_mut);
        assert_eq!(vec_mut, vec![1, 3, 5, 7]);
    }

    #[test]
    fn test_remove_many_with_retain_all_removed() {
        let vec = vec![2, 4, 6, 8];
        let mut vec_mut = vec.clone();
        // Keep only odd numbers (none exist, so remove all)
        let mutation = VecMutation::remove_many_with_retain(&vec, |_, &item| item % 2 != 0);
        mutation.apply(&mut vec_mut);
        assert_eq!(vec_mut, Vec::<i32>::new());
    }

    #[test]
    fn test_remove_many_with_retain_by_index() {
        let vec = vec!["a", "b", "c", "d", "e"];
        let mut vec_mut = vec.clone();
        // Keep items at odd indices (remove items at even indices)
        let mutation = VecMutation::remove_many_with_retain(&vec, |i, _| i % 2 != 0);
        mutation.apply(&mut vec_mut);
        assert_eq!(vec_mut, vec!["b", "d"]);
    }

    #[test]
    fn test_remove_many_with_retain_combined_criteria() {
        let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut vec_mut = vec.clone();
        // Keep items where value <= 5 OR index is odd
        let mutation =
            VecMutation::remove_many_with_retain(&vec, |i, &item| item <= 5 || i % 2 != 0);
        mutation.apply(&mut vec_mut);
        assert_eq!(vec_mut, vec![1, 2, 3, 4, 5, 6, 8, 10]);
    }

    #[test]
    fn test_remove_many_with_retain_complex_type() {
        #[derive(Debug, Clone, PartialEq)]
        struct Person {
            name: String,
            age: i32,
        }

        let vec = vec![
            Person {
                name: "Alice".to_string(),
                age: 25,
            },
            Person {
                name: "Bob".to_string(),
                age: 30,
            },
            Person {
                name: "Charlie".to_string(),
                age: 35,
            },
            Person {
                name: "Dave".to_string(),
                age: 40,
            },
        ];
        let mut vec_mut = vec.clone();
        // Keep people 30 or younger (remove people older than 30)
        let mutation = VecMutation::remove_many_with_retain(&vec, |_, person| person.age <= 30);
        mutation.apply(&mut vec_mut);
        assert_eq!(
            vec_mut,
            vec![
                Person {
                    name: "Alice".to_string(),
                    age: 25
                },
                Person {
                    name: "Bob".to_string(),
                    age: 30
                },
            ]
        );
    }

    #[test]
    fn test_remove_many_with_retain_first_and_last() {
        let vec = vec![1, 2, 3, 4, 5];
        let mut vec_mut = vec.clone();
        // Keep middle elements (remove first and last)
        let mutation = VecMutation::remove_many_with_retain(&vec, |i, _| i != 0 && i != 4);
        mutation.apply(&mut vec_mut);
        assert_eq!(vec_mut, vec![2, 3, 4]);
    }

    #[test]
    fn test_remove_many_with_retain_consecutive() {
        let vec = vec![1, 2, 3, 4, 5, 6, 7];
        let mut vec_mut = vec.clone();
        // Keep elements at indices < 2 or > 4 (remove indices 2, 3, 4)
        let mutation = VecMutation::remove_many_with_retain(&vec, |i, _| i < 2 || i > 4);
        mutation.apply(&mut vec_mut);
        assert_eq!(vec_mut, vec![1, 2, 6, 7]);
    }

    #[test]
    fn test_index_set_deduplication() {
        let index_set = IndexSet::from(vec![3, 1, 2, 1, 3, 2]);
        assert_eq!(&*index_set, &[1, 2, 3]);
    }

    #[test]
    fn test_index_set_sorting() {
        let index_set = IndexSet::from(vec![5, 2, 8, 1, 9]);
        assert_eq!(&*index_set, &[1, 2, 5, 8, 9]);
    }

    #[test]
    fn test_index_set_empty() {
        let index_set = IndexSet::from(vec![]);
        let empty: &[usize] = &[];
        assert_eq!(&*index_set, empty);
    }

    #[test]
    fn test_index_set_single() {
        let index_set = IndexSet::from(vec![42]);
        assert_eq!(&*index_set, &[42]);
    }

    #[test]
    fn test_sequential_mutations() {
        // Test applying multiple mutations in sequence
        let mut vec = vec![1, 2, 3];
        VecMutation::Push(4).apply(&mut vec);
        VecMutation::InsertOne(0, 0).apply(&mut vec);
        VecMutation::RemoveOne(2).apply(&mut vec);
        VecMutation::Update(1, 100).apply(&mut vec);
        assert_eq!(vec, vec![0, 100, 3, 4]);
    }
}
