use crate::{class::Class, format_class};
use dioxus::prelude::*;
use zino_core::SharedString;

/// A classic modal with a header and a body.
pub fn ModalCard(props: ModalCardProps) -> Element {
    let class = format_class!(props, "modal");
    let active_class = format_class!(props, active_class, "is-active");
    let close_class = format_class!(props, close_class, "delete");
    let container_class = if props.visible {
        format!("{class} {active_class}").into()
    } else {
        class
    };
    rsx! {
        div {
            class: "{container_class}",
            div { class: "modal-background" }
            div {
                class: "modal-card",
                header {
                    class: "modal-card-head",
                    div {
                        class: "modal-card-title",
                        "{props.title}"
                    }
                    button {
                        class: "{close_class}",
                        onclick: move |event| {
                            if let Some(handler) = props.on_close.as_ref() {
                                handler.call(event);
                            }
                        }
                    }
                }
                section {
                    class: "modal-card-body",
                    { props.children }
                }
            }
        }
    }
}

/// The [`ModalCard`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct ModalCardProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// A class to apply when the modal is visible.
    #[props(into)]
    pub active_class: Option<Class>,
    // A class to apply to the `close` button element.
    #[props(into)]
    pub close_class: Option<Class>,
    /// An event handler to be called when the `close` button is clicked.
    pub on_close: Option<EventHandler<MouseEvent>>,
    /// A flag to determine whether the modal is visible or not.
    #[props(default = false)]
    pub visible: bool,
    /// The title in the modal header.
    #[props(into)]
    pub title: SharedString,
    /// The model body to render within the component.
    children: Element,
}
