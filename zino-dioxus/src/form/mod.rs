//! Generic form controls.

use std::borrow::Cow;

mod button;
mod checkbox;
mod field;
mod file;
mod input;
mod progress;
mod radio;
mod select;
mod textarea;

#[cfg(feature = "clipboard")]
mod clipboard;

pub use button::{Button, ButtonProps, Buttons, ButtonsProps};
pub use checkbox::{Checkbox, CheckboxProps};
pub use field::{
    FormAddons, FormAddonsProps, FormField, FormFieldContainer, FormFieldContainerProps,
    FormFieldProps, FormGroup, FormGroupProps,
};
pub use file::{FileUpload, FileUploadProps};
pub use input::{Input, InputProps};
pub use progress::{Progress, ProgressProps};
pub use radio::{Radio, RadioProps};
pub use select::{DataSelect, DataSelectProps};
pub use textarea::{Textarea, TextareaProps};

#[cfg(feature = "clipboard")]
pub use clipboard::{CopyToClipboard, CopyToClipboardProps};

/// An interface for the data entries.
pub trait DataEntry {
    /// Returns the unique key.
    fn key(&self) -> Cow<'_, str>;

    /// Returns the value.
    fn value(&self) -> Cow<'_, str>;

    /// Returns the label.
    fn label(&self) -> Cow<'_, str>;
}

impl<'a> DataEntry for [&'a str; 2] {
    #[inline]
    fn key(&self) -> Cow<'_, str> {
        self[0].into()
    }

    #[inline]
    fn value(&self) -> Cow<'_, str> {
        self[0].into()
    }

    #[inline]
    fn label(&self) -> Cow<'_, str> {
        self[1].into()
    }
}

impl<T: ToString, U: ToString> DataEntry for (T, U) {
    #[inline]
    fn key(&self) -> Cow<'_, str> {
        self.0.to_string().into()
    }

    #[inline]
    fn value(&self) -> Cow<'_, str> {
        self.0.to_string().into()
    }

    #[inline]
    fn label(&self) -> Cow<'_, str> {
        self.1.to_string().into()
    }
}
