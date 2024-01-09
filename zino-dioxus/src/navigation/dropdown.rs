use crate::theme::Theme;
use dioxus::prelude::*;

/// An interactive dropdown menu for discoverable content.
pub fn Dropdown<'a>(cx: Scope<'a, DropdownProps<'a>>) -> Element {
    render! {
        div {}
    }
}

/// The [`Dropdown`] properties struct for the configuration of the component.
#[derive(Debug, PartialEq, Props)]
pub struct DropdownProps<'a> {
    /// Theme.
    theme: &'a Theme,
}
