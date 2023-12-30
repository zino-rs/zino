//! CSS classes for components.

use smallvec::SmallVec;
use std::{borrow::Cow, fmt};

/// A class type for dioxus components.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Class<'a> {
    /// Optional namespace.
    namespace: Option<&'a str>,
    /// A list of classes.
    classes: SmallVec<[&'a str; 5]>,
}

impl<'a> Class<'a> {
    /// Creates a new instance.
    #[inline]
    pub fn new(class: &'a str) -> Self {
        Self {
            namespace: None,
            classes: class.split_whitespace().collect(),
        }
    }

    /// Creates a new instance with the specific namespace.
    #[inline]
    pub fn with_namespace(namespace: &'a str, class: &'a str) -> Self {
        Self {
            namespace: (!namespace.is_empty()).then_some(namespace),
            classes: class.split_whitespace().collect(),
        }
    }

    /// Adds a class to the list, omitting any that are already present.
    #[inline]
    pub fn add(&mut self, class: &'a str) {
        if !(class.is_empty() || self.contains(class)) {
            self.classes.push(class);
        }
    }

    /// Removes a class from the list.
    #[inline]
    pub fn remove(&mut self, class: &str) {
        self.classes.retain(|s| s != &class)
    }

    /// Toggles a class in the list.
    #[inline]
    pub fn toggle(&mut self, class: &'a str) {
        if let Some(index) = self.classes.iter().position(|&s| s == class) {
            self.classes.remove(index);
        } else {
            self.classes.push(class);
        }
    }

    /// Replaces a class in the list with a new class.
    #[inline]
    pub fn replace(&mut self, class: &str, new_class: &'a str) {
        if let Some(index) = self.classes.iter().position(|&s| s == class) {
            self.classes[index] = new_class;
        }
    }

    /// Returns `true` if a given class has been added.
    #[inline]
    pub fn contains(&self, class: &str) -> bool {
        self.classes.iter().any(|&s| s == class)
    }

    /// Returns `true` if the class list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    /// Returns the namespace.
    #[inline]
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.filter(|s| !s.is_empty())
    }

    /// Formats `self` as a `Cow<str>`.
    pub fn format(&self) -> Cow<'_, str> {
        let classes = self.classes.as_slice();
        if let Some(namespace) = self.namespace() {
            let class = if let [class] = classes {
                [namespace, class].join("-")
            } else {
                classes
                    .iter()
                    .filter(|s| !s.is_empty())
                    .map(|s| [namespace, s].join("-"))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            Cow::Owned(class)
        } else if let [class] = classes {
            Cow::Borrowed(class)
        } else {
            Cow::Owned(classes.join(" "))
        }
    }
}

impl<'a> From<&'a str> for Class<'a> {
    #[inline]
    fn from(class: &'a str) -> Self {
        Self::new(class)
    }
}

impl<'a> From<Vec<&'a str>> for Class<'a> {
    #[inline]
    fn from(classes: Vec<&'a str>) -> Self {
        Self {
            namespace: None,
            classes: SmallVec::from_vec(classes),
        }
    }
}

impl<'a, const N: usize> From<[&'a str; N]> for Class<'a> {
    #[inline]
    fn from(classes: [&'a str; N]) -> Self {
        Self {
            namespace: None,
            classes: SmallVec::from_slice(&classes),
        }
    }
}

impl<'a> fmt::Display for Class<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let format = self.format();
        write!(f, "{format}")
    }
}

/// Formats the class with a default value.
#[macro_export]
macro_rules! format_class {
    ($cx:ident, $default_class:expr) => {
        $cx.props
            .class
            .as_ref()
            .map(Class::format)
            .unwrap_or_else(|| $default_class.into())
    };
}
