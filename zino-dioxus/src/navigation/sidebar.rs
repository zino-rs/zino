use crate::{class::Class, format_class};
use dioxus::prelude::*;

/// A vertical menu used in the navigation aside.
pub fn Sidebar<'a>(cx: Scope<'a, SidebarProps<'a>>) -> Element {
    let class = format_class!(cx, "sidebar");
    render! {
        div {
            class: "{class}",
            &cx.props.children
        }
    }
}

/// The [`Sidebar`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct SidebarProps<'a> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// The children to render within the component.
    children: Element<'a>,
}
