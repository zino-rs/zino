use crate::theme::Theme;
use dioxus::prelude::*;

/// An interactive dropdown menu for discoverable content.
pub fn DropdownMenu<'a>(cx: Scope<'a, DropdownMenuProps<'a>>) -> Element {
    render! {
        div {}
    }
}

/// The [`DropdownMenu`] properties struct for the configuration of the menu.
#[derive(Debug, PartialEq, Props)]
pub struct DropdownMenuProps<'a> {
    /// Theme.
    theme: &'a Theme,
}
