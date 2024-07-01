use crate::class::Class;
use dioxus::prelude::*;
use zino_core::SharedString;

/// The classic button in different colors, sizes, and states.
pub fn Button(props: ButtonProps) -> Element {
    rsx! {
        button {
            class: props.class,
            class: if !props.color.is_empty() { "is-{props.color}" },
            class: if !props.theme.is_empty() { "is-{props.theme}" },
            class: if !props.size.is_empty() { "is-{props.size}" },
            class: if !props.state.is_empty() { "is-{props.state}" },
            class: if props.responsive { "is-responsive" },
            class: if props.fullwidth { "is-fullwidth" },
            class: if props.outlined { "is-outlined" },
            class: if props.inverted { "is-inverted" },
            class: if props.rounded { "is-rounded" },
            onclick: move |event| {
                if let Some(handler) = props.on_click.as_ref() {
                    event.stop_propagation();
                    handler.call(event);
                }
            },
            disabled: "{props.disabled}",
            ..props.attributes,
            { props.children }
        }
    }
}

/// The [`Button`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct ButtonProps {
    /// The class attribute for the component.
    #[props(into, default = "button".into())]
    pub class: Class,
    /// The color of the button: `white` | `light` | `dark` | `black` | `text` | `ghost`
    /// | `primary` | `link` | `info` | `success` | `warning` | `danger`.
    #[props(into, default)]
    pub color: SharedString,
    /// The theme of the button: `light` | `dark`.
    #[props(into, default)]
    pub theme: SharedString,
    /// The size of the button: `small` | `normal` | `medium` | `large`.
    #[props(into, default)]
    pub size: SharedString,
    /// The state of the button: `hover` | `focus` | `active` | `loading` | `static`.
    #[props(into, default)]
    pub state: SharedString,
    /// A flag to determine whether the button size is responsive or not.
    #[props(default)]
    pub responsive: bool,
    /// A flag to determine whether the button is a fullwidth block or not.
    #[props(default)]
    pub fullwidth: bool,
    /// A flag to determine whether the button has an outline or not.
    #[props(default)]
    pub outlined: bool,
    /// A flag to determine whether the button color is inverted or not.
    #[props(default)]
    pub inverted: bool,
    /// A flag to determine whether the button is rounded or not.
    #[props(default)]
    pub rounded: bool,
    /// A flag to determine whether the button is disabled or not.
    #[props(default)]
    pub disabled: bool,
    /// An event handler to be called when the button is clicked.
    pub on_click: Option<EventHandler<MouseEvent>>,
    /// Spreading the props of the `button` element.
    #[props(extends = button)]
    attributes: Vec<Attribute>,
    /// The children to render within the component.
    children: Element,
}
