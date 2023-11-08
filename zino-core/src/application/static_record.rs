/// A static record type.
#[derive(Debug, Default)]
pub struct StaticRecord<T> {
    /// Inner container.
    inner: Vec<(&'static str, T)>,
}

impl<T> StaticRecord<T> {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Appends an entry to the back of a collection.
    #[inline]
    pub fn add(&mut self, key: &'static str, value: T) {
        self.inner.push((key, value));
    }

    /// Searches for the key and returns its value.
    #[inline]
    pub fn find(&self, key: &str) -> Option<&T> {
        self.inner
            .iter()
            .find_map(|(field, value)| (field == &key).then_some(value))
    }
}

impl<T> IntoIterator for StaticRecord<T> {
    type Item = (&'static str, T);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
