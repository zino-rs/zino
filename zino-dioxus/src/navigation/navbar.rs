use crate::{class::Class, extension::VNodeExt, format_class};
use dioxus::prelude::*;
use dioxus_router::components::{IntoRoutable, Link};

/// A responsive navigation header.
pub fn Navbar(props: NavbarProps) -> Element {
    let children = props.children.as_ref()?;
    let class = format_class!(props, "navbar is-link");
    if children.has_component("NavbarBrand") {
        rsx! {
            nav {
                class: "{class}",
                { children }
            }
        }
    } else {
        rsx! {
            nav {
                class: "{class}",
                NavbarMenu {
                    { children }
                }
            }
        }
    }
}

/// The [`Navbar`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The children to render within the component.
    children: Element,
}

/// A container for the logo and optionally some links or icons.
pub fn NavbarBrand(props: NavbarBrandProps) -> Element {
    let class = format_class!(props, "navbar-brand");
    rsx! {
        div {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`NavbarBrand`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarBrandProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The children to render within the component.
    children: Element,
}

/// A horizontal menu used in the navigation header.
pub fn NavbarMenu(props: NavbarMenuProps) -> Element {
    let class = format_class!(props, "navbar-menu is-active");
    rsx! {
        div {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`NavbarMenu`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarMenuProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The children to render within the component.
    children: Element,
}

/// The left section of the navbar menu.
pub fn NavbarStart(props: NavbarStartProps) -> Element {
    let class = format_class!(props, "navbar-start");
    rsx! {
        div {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`NavbarStart`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarStartProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The children to render within the component.
    children: Element,
}

/// The middle section of the navbar menu.
pub fn NavbarCenter(props: NavbarCenterProps) -> Element {
    let class = format_class!(props, "navbar-center");
    rsx! {
        div {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`NavbarCenter`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarCenterProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The children to render within the component.
    children: Element,
}

/// The right section of the navbar menu.
pub fn NavbarEnd(props: NavbarEndProps) -> Element {
    let class = format_class!(props, "navbar-end");
    rsx! {
        div {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`NavbarEnd`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarEndProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The children to render within the component.
    children: Element,
}

/// A link to navigate to another route in the navigation header.
pub fn NavbarLink(props: NavbarLinkProps) -> Element {
    let class = format_class!(props, "navbar-item");
    let active_class = format_class!(props, "is-active");
    rsx! {
        Link {
            class: "{class}",
            active_class: "{active_class}",
            to: props.to.clone(),
            { props.children }
        }
    }
}

/// The [`NavbarLink`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarLinkProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// A class to apply to the generate HTML anchor tag if the `target` route is active.
    #[props(into)]
    pub active_class: Option<Class>,
    /// The navigation target. Roughly equivalent to the href attribute of an HTML anchor tag.
    #[props(into)]
    pub to: IntoRoutable,
    /// The children to render within the component.
    children: Element,
}

/// A container for each single item of the navbar.
pub fn NavbarItem(props: NavbarItemProps) -> Element {
    let class = format_class!(props, "navbar-item");
    rsx! {
        div {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`NavbarItem`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarItemProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The children to render within the component.
    children: Element,
}
