use crate::theme::Theme;
use dioxus::prelude::*;

/// A vertical menu used in the navigation aside.
pub fn SidebarMenu<'a>(cx: Scope<'a, SidebarMenuProps<'a>>) -> Element {
    render! {
        div {}
    }
}

/// The [`SidebarMenu`] properties struct for the configuration of the menu.
#[derive(Debug, PartialEq, Props)]
pub struct SidebarMenuProps<'a> {
    /// Theme.
    theme: &'a Theme,
}
