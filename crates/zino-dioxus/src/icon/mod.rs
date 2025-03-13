//! Icon fonts or SVG icon shapes.

use crate::class::Class;
use dioxus::prelude::*;
use dioxus_free_icons::IconShape;

/// A container for any type of icon fonts.
pub fn Icon(props: IconProps) -> Element {
    if props.icon_class.is_empty() {
        rsx! {
            span {
                class: props.class,
                { props.children }
            }
        }
    } else {
        rsx! {
            span {
                class: props.class,
                i {
                    class: props.icon_class,
                }
            }
        }
    }
}

/// The [`Icon`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct IconProps {
    /// The class attribute for the component.
    #[props(into, default = "icon")]
    pub class: Class,
    /// The class to apply to the `<i>` element for a icon font.
    #[props(into, default)]
    pub icon_class: Class,
    /// The children to render within the component.
    children: Element,
}

/// A container for a SVG icon.
pub fn SvgIcon<T: IconShape + Clone + PartialEq + 'static>(props: SvgIconProps<T>) -> Element {
    let width = props.width;
    let height = props.height.unwrap_or(width);
    let style = if props.intrinsic {
        format!("width:{width}px;height:{height}px")
    } else {
        String::new()
    };
    rsx! {
        span {
            class: props.class,
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
    #[props(into, default = "icon")]
    pub class: Class,
    /// The icon shape to use.
    pub shape: T,
    /// The width of the `<svg>` element. Defaults to 20.
    #[props(default = 20)]
    pub width: u32,
    /// The height of the `<svg>` element.
    #[props(into)]
    pub height: Option<u32>,
    /// A flag to use the instrinsic size for the icon.
    #[props(default)]
    pub intrinsic: bool,
}

/// A wrapper for combining an icon with text.
pub fn IconText(props: IconTextProps) -> Element {
    rsx! {
        span {
            class: "{props.class}",
            { props.children }
        }
    }
}

/// The [`IconText`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct IconTextProps {
    /// The class attribute for the component.
    #[props(into, default = "icon-text")]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}
