use crate::class::Class;
use dioxus::prelude::*;

/// The mutually exclusive radio buttons in their native format.
pub fn Radio(props: RadioProps) -> Element {
    rsx! {
        label {
            class: props.label_class,
            input {
                class: props.class,
                r#type: "radio",
                onchange: move |event| {
                    if let Some(handler) = props.on_change.as_ref() {
                        handler.call(event.value());
                    }
                },
                ..props.attributes,
            }
            { props.children }
        }
    }
}

/// The [`Radio`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct RadioProps {
    /// The class attribute for the component.
    #[props(into, default = "radio")]
    pub class: Class,
    /// A class to apply to the `label` element.
    #[props(into, default = "radio")]
    pub label_class: Class,
    /// An event handler to be called when the radio button is checked.
    pub on_change: Option<EventHandler<String>>,
    /// Spreading the props of the `input` element.
    #[props(extends = input)]
    attributes: Vec<Attribute>,
    /// The children to render within the component.
    children: Element,
}
