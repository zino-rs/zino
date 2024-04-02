//! Icon fonts or SVG icon shapes.

use crate::{class::Class, format_class};
use dioxus::prelude::*;
use dioxus_free_icons::IconShape;

/// A container for any type of icon fonts.
pub fn Icon(props: IconProps) -> Element {
    let class = format_class!(props, "icon");
    if let Some(icon) = props.icon_class.as_ref() {
        let icon_class = icon.format();
        rsx! {
            span {
                class: "{class}",
                i {
                    class: "{icon_class}"
                }
            }
        }
    } else {
        rsx! {
            span {
                class: "{class}",
                { props.children }
            }
        }
    }
}

/// The [`Icon`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct IconProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The class to apply to the `<i>` element for a icon font.
    #[props(into)]
    pub icon_class: Option<Class>,
    /// The children to render within the component.
    children: Element,
}

/// A container for a SVG icon.
pub fn SvgIcon<T: IconShape + Clone + PartialEq + 'static>(props: SvgIconProps<T>) -> Element {
    let class = format_class!(props, "icon");
    let width = props.width;
    let height = props.height.unwrap_or(width);
    let style = if props.intrinsic {
        format!("width:{width}px;height:{height}px")
    } else {
        String::new()
    };
    rsx! {
        span {
            class: "{class}",
            style: "{style}",
            dioxus_free_icons::Icon {
                icon: props.shape,
                width: width,
                height: height,
            }
        }
    }
}

/// The [`SvgIcon`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct SvgIconProps<T: IconShape + Clone + PartialEq + 'static> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The icon shape to use.
    pub shape: T,
    /// The width of the `<svg>` element. Defaults to 20.
    #[props(default = 20)]
    pub width: u32,
    /// The height of the `<svg>` element.
    #[props(into)]
    pub height: Option<u32>,
    /// A flag to use the instrinsic size for the icon.
    #[props(default = false)]
    pub intrinsic: bool,
}

/// A wrapper for combining an icon with text.
pub fn IconText(props: IconTextProps) -> Element {
    let class = format_class!(props, "icon-text");
    rsx! {
        span {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`IconText`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct IconTextProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The children to render within the component.
    children: Element,
}
