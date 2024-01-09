use crate::{class::Class, format_class};
use dioxus::prelude::*;

/// A responsive, fixed-width container with the `max-width` changes at each breakpoint.
pub fn Container<'a>(cx: Scope<'a, ContainerProps<'a>>) -> Element {
    let class = format_class!(cx, "container");
    render! {
        main {
            class: "{class}",
            &cx.props.children
        }
    }
}

/// The [`FluidContainer`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct ContainerProps<'a> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// The children to render within the component.
    children: Element<'a>,
}

/// A full width container spanning the entire width of the viewport.
pub fn FluidContainer<'a>(cx: Scope<'a, FluidContainerProps<'a>>) -> Element {
    let class = format_class!(cx, "container-fluid");
    render! {
        main {
            class: "{class}",
            &cx.props.children
        }
    }
}

/// The [`FluidContainer`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct FluidContainerProps<'a> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// The children to render within the component.
    children: Element<'a>,
}

/// A container rendered as the `<main>` element.
pub fn MainContainer<'a>(cx: Scope<'a, MainContainerProps<'a>>) -> Element {
    let class = format_class!(cx, "container-fluid px-3 my-3");
    render! {
        main {
            class: "{class}",
            &cx.props.children
        }
    }
}

/// The [`MainContainer`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct MainContainerProps<'a> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// The children to render within the component.
    children: Element<'a>,
}
