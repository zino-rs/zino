use crate::theme::Theme;
use dioxus::prelude::*;

/// A horizontal menu used in the navigation header.
pub fn NavbarMenu<'a>(cx: Scope<'a, NavbarMenuProps<'a>>) -> Element {
    render! {
        div {}
    }
}

/// The [`NavbarMenu`] properties struct for the configuration of the menu.
#[derive(Debug, PartialEq, Props)]
pub struct NavbarMenuProps<'a> {
    /// Theme.
    theme: &'a Theme,
}
