use crate::{class::Class, format_class};
use dioxus::prelude::*;

/// A vertical menu used in the navigation aside.
pub fn Sidebar(props: SidebarProps) -> Element {
    let class = format_class!(props, "sidebar");
    rsx! {
        div {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`Sidebar`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct SidebarProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The children to render within the component.
    children: Element,
}
