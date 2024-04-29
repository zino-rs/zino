use crate::class::Class;
use dioxus::prelude::*;
use zino_core::SharedString;

/// A classic modal with a header and a body.
pub fn ModalCard(props: ModalCardProps) -> Element {
    let class = props.class;
    let active_class = props.active_class;
    let title_class = props.title_class;
    let close_class = props.close_class;
    let container_class = if props.visible {
        format!("{class} {active_class}")
    } else {
        class.to_string()
    };
    let size = match props.size.as_ref() {
        "small" => 25,
        "medium" => 50,
        "large" => 75,
        _ => 40,
    };
    rsx! {
        div {
            class: "{container_class}",
            style: "--bulma-modal-content-width:{size}rem",
            div { class: "modal-background" }
            div {
                class: "modal-card",
                header {
                    class: "modal-card-head",
                    div {
                        class: "modal-card-title {title_class}",
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
    #[props(into, default = "modal".into())]
    pub class: Class,
    /// A class to apply when the modal is visible.
    #[props(into, default = "is-active".into())]
    pub active_class: Class,
    // A class to apply to the modal title.
    #[props(into, default)]
    pub title_class: Class,
    /// A class to apply to the `close` button element.
    #[props(into, default = "delete".into())]
    pub close_class: Class,
    /// An event handler to be called when the `close` button is clicked.
    pub on_close: Option<EventHandler<MouseEvent>>,
    /// A flag to determine whether the modal is visible or not.
    #[props(default)]
    pub visible: bool,
    /// The size of the modal: `small` | "normal" | `medium` | "large".
    #[props(into, default = "normal".into())]
    pub size: SharedString,
    /// The title in the modal header.
    #[props(into)]
    pub title: SharedString,
    /// The model body to render within the component.
    children: Element,
}
