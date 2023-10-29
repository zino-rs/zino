use crate::theme::Theme;
use dioxus::prelude::*;

/// A vertical menu used in the navigation aside.
pub fn Sidebar<'a>(cx: Scope<'a, SidebarProps<'a>>) -> Element {
    render! {
        div {}
    }
}

/// The [`Sidebar`] properties struct for the configuration of the component.
#[derive(Debug, PartialEq, Props)]
pub struct SidebarProps<'a> {
    /// Theme.
    theme: &'a Theme,
}
