use crate::class::Class;
use dioxus::prelude::*;
use zino_core::SharedString;

/// A small tag label in different colors and sizes.
pub fn Tag(props: TagProps) -> Element {
    rsx! {
        button {
            class: "{props.class}",
            class: if !props.color.is_empty() { "is-{props.color}" },
            class: if !props.theme.is_empty() { "is-{props.theme}" },
            class: if !props.size.is_empty() { "is-{props.size}" },
            class: if props.hoverable { "is-hoverable" },
            class: if props.rounded { "is-rounded" },
            onclick: move |event| {
                if let Some(handler) = props.on_click.as_ref() {
                    handler.call(event);
                }
            },
            { props.children }
        }
    }
}

/// The [`Tag`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct TagProps {
    /// The class attribute for the component.
    #[props(into, default = "tag")]
    pub class: Class,
    /// The color of the tag: `white` | `light` | `dark` | `black`
    /// | `primary` | `link` | `info` | `success` | `warning` | `danger`.
    #[props(into, default)]
    pub color: SharedString,
    /// The theme of the tag: `light`.
    #[props(into, default)]
    pub theme: SharedString,
    /// The size of the tag: `normal` | `medium` | `large`.
    #[props(into, default)]
    pub size: SharedString,
    /// A flag to determine whether the tag is hoverable or not.
    #[props(default)]
    pub hoverable: bool,
    /// A flag to determine whether the tag is rounded or not.
    #[props(default)]
    pub rounded: bool,
    /// An event handler to be called when the tag is clicked.
    pub on_click: Option<EventHandler<MouseEvent>>,
    /// The children to render within the component.
    children: Element,
}

/// A list of tags.
pub fn Tags(props: TagsProps) -> Element {
    let justify = props.justify;
    rsx! {
        div {
            class: "{props.class}",
            class: if !justify.is_empty() { "is-justify-content-{justify}" },
            class: if props.addons { "has-addons" },
            { props.children }
        }
    }
}

/// The [`Tags`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct TagsProps {
    /// The class attribute for the component.
    #[props(into, default = "tags")]
    pub class: Class,
    /// The `justify-content` value: `flex-start` | `flex-end` | `center` | `space-between`
    /// | `space-around` | `space-evenly` | `start` | `end` | `left` | `right`.
    #[props(into, default)]
    pub justify: SharedString,
    /// A flag to determine whether the tags are attached together or not.
    #[props(default)]
    pub addons: bool,
    /// The children to render within the component.
    children: Element,
}
