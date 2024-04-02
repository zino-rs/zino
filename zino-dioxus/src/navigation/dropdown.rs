use crate::theme::Theme;
use dioxus::prelude::*;

/// An interactive dropdown menu for discoverable content.
pub fn Dropdown(_props: DropdownProps) -> Element {
    rsx! {
        div {}
    }
}

/// The [`Dropdown`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct DropdownProps {
    /// Theme.
    theme: Theme,
}
