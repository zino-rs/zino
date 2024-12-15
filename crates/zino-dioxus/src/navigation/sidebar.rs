use crate::class::Class;
use dioxus::prelude::*;

/// A vertical menu used in the navigation aside.
pub fn Sidebar(props: SidebarProps) -> Element {
    rsx! {
        aside {
            class: props.class,
            { props.children }
        }
    }
}

/// The [`Sidebar`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct SidebarProps {
    /// The class attribute for the component.
    #[props(into, default = "sidebar")]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}
