//! CSS classes for components.

use dioxus_core::{AttributeValue, prelude::*};
use smallvec::SmallVec;
use std::{borrow::Cow, fmt};

/// A class type for dioxus components.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Class {
    /// Optional namespace.
    namespace: Option<&'static str>,
    /// A list of classes.
    classes: SmallVec<[&'static str; 4]>,
}

impl Class {
    /// Creates a new instance.
    #[inline]
    pub fn new(class: &'static str) -> Self {
        Self {
            namespace: None,
            classes: class.split_whitespace().collect(),
        }
    }

    /// Creates a new instance with the specific namespace.
    #[inline]
    pub fn with_namespace(namespace: &'static str, class: &'static str) -> Self {
        Self {
            namespace: (!namespace.is_empty()).then_some(namespace),
            classes: class.split_whitespace().collect(),
        }
    }

    /// Adds a class to the list, omitting any that are already present.
    #[inline]
    pub fn add(&mut self, class: &'static str) {
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
    pub fn toggle(&mut self, class: &'static str) {
        if let Some(index) = self.classes.iter().position(|&s| s == class) {
            self.classes.remove(index);
        } else {
            self.classes.push(class);
        }
    }

    /// Replaces a class in the list with a new class.
    #[inline]
    pub fn replace(&mut self, class: &str, new_class: &'static str) {
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
        if self.classes.is_empty() {
            return Cow::Borrowed("");
        }

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

impl From<&'static str> for Class {
    #[inline]
    fn from(class: &'static str) -> Self {
        Self::new(class)
    }
}

impl From<Vec<&'static str>> for Class {
    #[inline]
    fn from(classes: Vec<&'static str>) -> Self {
        Self {
            namespace: None,
            classes: SmallVec::from_vec(classes),
        }
    }
}

impl<const N: usize> From<[&'static str; N]> for Class {
    #[inline]
    fn from(classes: [&'static str; N]) -> Self {
        Self {
            namespace: None,
            classes: SmallVec::from_slice(&classes),
        }
    }
}

impl fmt::Display for Class {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let format = self.format();
        write!(f, "{format}")
    }
}

impl IntoAttributeValue for Class {
    #[inline]
    fn into_value(self) -> AttributeValue {
        AttributeValue::Text(self.format().into_owned())
    }
}
