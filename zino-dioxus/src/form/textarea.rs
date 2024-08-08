use crate::class::Class;
use dioxus::prelude::*;

/// The multiline textarea and its variations.
pub fn Textarea(props: TextareaProps) -> Element {
    rsx! {
        textarea {
            class: props.class,
            value: props.initial_value,
            ..props.attributes,
            onchange: move |event| async move {
                if let Some(handler) = props.on_change.as_ref() {
                    handler.call(event.value());
                }
            },
            oninput: move |event| async move {
                if let Some(handler) = props.on_input.as_ref() {
                    handler.call(event.value());
                }
            }
        }
    }
}

/// The [`Textarea`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct TextareaProps {
    /// The class attribute for the component.
    #[props(into, default = "textarea".into())]
    pub class: Class,
    /// The initial value of the textarea.
    #[props(into, default)]
    pub initial_value: String,
    /// An event handler to be called when the textarea state is changed.
    pub on_change: Option<EventHandler<String>>,
    /// An event handler to be called when inputing.
    pub on_input: Option<EventHandler<String>>,
    /// Spreading the props of the `textarea` element.
    #[props(extends = textarea)]
    attributes: Vec<Attribute>,
}
