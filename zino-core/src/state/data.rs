use parking_lot::RwLock;
use std::mem;

/// Data wrapper.
#[derive(Debug, Default)]
pub struct Data<T>(T);

impl<T> Data<T> {
    /// Creates a new instance.
    #[inline]
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Unwraps to the contained value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Clone> Data<T> {
    /// Returns a copy of the contained value.
    #[inline]
    pub fn get(&self) -> T {
        self.0.clone()
    }
}

/// Shared data wrapper.
#[derive(Debug, Default)]
pub struct SharedData<T>(RwLock<T>);

impl<T> SharedData<T> {
    /// Creates a new instance.
    #[inline]
    pub fn new(value: T) -> Self {
        Self(RwLock::new(value))
    }

    /// Sets the contained value.
    #[inline]
    pub fn set(&self, value: T) {
        *self.0.write() = value;
    }

    /// Replaces the contained value with `value`, and returns the old contained value.
    #[inline]
    pub fn replace(&self, value: T) -> T {
        mem::replace(&mut self.0.write(), value)
    }

    /// Unwraps to the contained value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }
}

impl<T: Clone> SharedData<T> {
    /// Returns a copy of the contained value.
    #[inline]
    pub fn get(&self) -> T {
        self.0.read().clone()
    }
}
