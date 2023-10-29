//! Icon fonts or SVG icon shapes.

use crate::{class::Class, format_class};
use dioxus::prelude::*;

/// A container for any type of icon fonts or SVG icon shapes.
pub fn Icon<'a>(cx: Scope<'a, IconProps<'a>>) -> Element {
    let class = format_class!(cx, "icon");
    if let Some(icon) = cx.props.icon_class.as_ref() {
        let icon_class = icon.format();
        render! {
            span {
                class: "{class}",
                i {
                    class: "{icon_class}"
                }
            }
        }
    } else {
        render! {
            span {
                class: "{class}",
                &cx.props.children
            }
        }
    }
}

/// The [`Icon`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct IconProps<'a> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// The class to apply to the `<i>` element for a icon font.
    #[props(into)]
    pub icon_class: Option<Class<'a>>,
    /// The children to render within the component.
    children: Element<'a>,
}

/// A wrapper for combining an icon with text.
pub fn IconText<'a>(cx: Scope<'a, IconTextProps<'a>>) -> Element {
    let class = format_class!(cx, "icon-text");
    render! {
        span {
            class: "{class}",
            &cx.props.children
        }
    }
}

/// The [`IconText`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct IconTextProps<'a> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// The children to render within the component.
    children: Element<'a>,
}
