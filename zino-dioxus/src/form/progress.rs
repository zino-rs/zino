use crate::class::Class;
use dioxus::prelude::*;
use zino_core::SharedString;

/// Native HTML progress bars.
pub fn Progress(props: ProgressProps) -> Element {
    rsx! {
        progress {
            class: props.class,
            class: if !props.color.is_empty() { "is-{props.color}" },
            class: if !props.size.is_empty() { "is-{props.size}" },
            ..props.attributes,
            { props.children }
        }
    }
}

/// The [`Progress`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct ProgressProps {
    /// The class attribute for the component.
    #[props(into, default = "progress".into())]
    pub class: Class,
    /// The color of the button: `primary` | `link` | `info` | `success` | `warning` | `danger`.
    #[props(into, default)]
    pub color: SharedString,
    /// The size of the button: `small` | `normal` | `medium` | `large`.
    #[props(into, default)]
    pub size: SharedString,
    /// Spreading the props of the `input` element.
    #[props(extends = input)]
    attributes: Vec<Attribute>,
    /// The children to render within the component.
    children: Element,
}
