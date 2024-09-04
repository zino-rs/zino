use crate::class::Class;
use dioxus::prelude::*;

/// The 2-state checkbox in its native format.
pub fn Checkbox(props: CheckboxProps) -> Element {
    rsx! {
        label {
            class: props.label_class,
            input {
                class: props.class,
                r#type: "checkbox",
                ..props.attributes,
                onchange: move |event| {
                    if let Some(handler) = props.on_change.as_ref() {
                        handler.call(event.value());
                    }
                }
            }
            { props.children }
        }
    }
}

/// The [`Checkbox`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct CheckboxProps {
    /// The class attribute for the component.
    #[props(into, default = "checkbox".into())]
    pub class: Class,
    /// A class to apply to the `label` element.
    #[props(into, default = "checkbox".into())]
    pub label_class: Class,
    /// An event handler to be called when the checkbox's state is changed.
    pub on_change: Option<EventHandler<String>>,
    /// Spreading the props of the `input` element.
    #[props(extends = input)]
    attributes: Vec<Attribute>,
    /// The children to render within the component.
    children: Element,
}
