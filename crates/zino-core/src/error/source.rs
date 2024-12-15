use super::Error;

/// An iterator of source errors created by [`sources()`](Error::sources).
pub struct Source<'a> {
    /// Next source error.
    next: Option<&'a Error>,
}

impl<'a> Source<'a> {
    /// Creates a new instance.
    #[inline]
    pub(super) fn new(error: &'a Error) -> Self {
        Self { next: Some(error) }
    }
}

impl<'a> Iterator for Source<'a> {
    type Item = &'a Error;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let error = self.next?;
        self.next = error.source();
        Some(error)
    }
}
