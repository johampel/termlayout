/// Decorator for an iterator that allows taking back previously emitted items.
///
/// # Example
/// ```rust
/// use termlayout::ext::TakeBackIterator;
///
/// let data = vec![1, 2, 3];
/// let mut iter = TakeBackIterator::new(data.into_iter());
///
/// assert_eq!(iter.next(), Some(1));
/// assert_eq!(iter.next(), Some(2));
/// iter.take_back(4711);
/// assert_eq!(iter.next(), Some(4711));
/// assert_eq!(iter.next(), Some(3));
/// assert_eq!(iter.next(), None);
/// ```
pub struct TakeBackIterator<T>
where
    T: Iterator,
{
    iterator: T,
    next: Option<T::Item>,
}

impl<T> TakeBackIterator<T>
where
    T: Iterator,
{
    /// Creates a new `TakeBackIterator` from the given iterator.
    ///
    /// # Parameters
    /// - `iterator`: The iterator to wrap.
    ///
    /// # Returns
    /// A new `TakeBackIterator` instance.
    pub fn new(iterator: T) -> Self {
        Self {
            iterator,
            next: None,
        }
    }

    /// Takes back a previously emitted item, so it is re-fetched with the next call.
    /// The function panics if this is called twice for one next call.
    ///
    /// # Parameters
    /// - `next`: The text to take back. If empty, it will not place an item.
    ///
    /// # Panics
    /// If `take_back` is called twice for one next call.
    pub fn take_back(&mut self, next: T::Item) {
        assert!(self.next.is_none(), "take_back() called twice");
        self.next = Some(next);
    }
}

impl<T> From<T> for TakeBackIterator<T>
where
    T: Iterator,
{
    fn from(iterator: T) -> Self {
        Self::new(iterator)
    }
}

impl<T> Iterator for TakeBackIterator<T>
where
    T: Iterator,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_some() {
            return self.next.take();
        }
        self.iterator.next()
    }
}
