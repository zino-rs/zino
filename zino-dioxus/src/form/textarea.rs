use crate::class::Class;
use dioxus::prelude::*;

/// The multiline textarea and its variations.
pub fn Textarea(props: TextareaProps) -> Element {
    rsx! {
        textarea {
            class: props.class,
            ..props.attributes,
        }
    }
}

/// The [`Textarea`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct TextareaProps {
    /// The class attribute for the component.
    #[props(into, default = "textarea".into())]
    pub class: Class,
    /// Spreading the props of the `textarea` element.
    #[props(extends = textarea)]
    attributes: Vec<Attribute>,
}
