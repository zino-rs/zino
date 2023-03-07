use super::Error;

/// Iterator of a chain of source errors created by [`chain()`](Error::chain).
pub struct Chain<'a> {
    /// Next source error.
    next: Option<&'a Error>,
}

impl<'a> Chain<'a> {
    /// Creates a new instance.
    #[inline]
    pub(super) fn new(error: &'a Error) -> Self {
        Self { next: Some(error) }
    }
}

impl<'a> Iterator for Chain<'a> {
    type Item = &'a Error;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let error = self.next?;
        self.next = error.source();
        Some(error)
    }
}
