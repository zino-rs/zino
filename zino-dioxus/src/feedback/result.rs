use crate::class::Class;
use dioxus::prelude::*;
use zino_core::SharedString;

/// Global messages as feedback in response to user operations.
pub fn OperationResult<T, E>(props: OperationResultProps<T, E>) -> Element
where
    T: Clone + PartialEq + 'static,
    E: Clone + PartialEq + 'static,
{
    if !props.visible {
        return None;
    }
    match props.future {
        Some(Ok(data)) => {
            if let Some(handler) = props.on_success.as_ref() {
                handler.call(data);
            }
            rsx! {
                div {
                    class: props.class,
                    class: "is-success",
                    position: "fixed",
                    top: "4rem",
                    right: "0.75rem",
                    z_index: 9999,
                    if !props.title.is_empty() {
                        div {
                            class: "message-header",
                            span { { props.title } }
                            button {
                                r#type: "button",
                                class: props.close_class,
                                onclick: move |_event| {
                                    if let Some(handler) = props.on_close.as_ref() {
                                        handler.call(false);
                                    }
                                }
                            }
                        }
                    }
                    div {
                        class: "message-body",
                        "{props.success}"
                    }
                }
            }
        }
        Some(Err(err)) => {
            if let Some(handler) = props.on_error.as_ref() {
                handler.call(err);
            }
            rsx! {
                div {
                    class: props.class,
                    class: "is-danger",
                    position: "fixed",
                    top: "4rem",
                    right: "0.75rem",
                    z_index: 9999,
                    if !props.title.is_empty() {
                        div {
                            class: "message-header",
                            span { { props.title } }
                            button {
                                r#type: "button",
                                class: props.close_class,
                                onclick: move |_event| {
                                    if let Some(handler) = props.on_close.as_ref() {
                                        handler.call(false);
                                    }
                                }
                            }
                        }
                    }
                    div {
                        class: "message-body",
                        span { "{props.error}" }
                    }
                }
            }
        }
        None => {
            if let Some(handler) = props.on_loading.as_ref() {
                handler.call(());
            }
            if props.loading.is_empty() {
                None
            } else {
                rsx! {
                    div {
                        class: props.class,
                        class: "is-warning",
                        position: "fixed",
                        top: "4rem",
                        right: "0.75rem",
                        z_index: 99,
                        if !props.title.is_empty() {
                            div {
                                class: "message-header",
                                span { { props.title } }
                                button {
                                    r#type: "button",
                                    class: props.close_class,
                                    onclick: move |_event| {
                                        if let Some(handler) = props.on_close.as_ref() {
                                            handler.call(false);
                                        }
                                    }
                                }
                            }
                        }
                        div {
                            class: "message-body",
                            span { { props.loading } }
                        }
                    }
                }
            }
        }
    }
}

/// The [`OperationResult`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct OperationResultProps<T: Clone + PartialEq + 'static, E: Clone + PartialEq + 'static> {
    /// The class attribute for the component.
    #[props(into, default = "message".into())]
    pub class: Class,
    /// A class to apply to the `close` button element.
    #[props(into, default = "delete".into())]
    pub close_class: Class,
    /// A future value which represents the result of user operations.
    pub future: Option<Result<T, E>>,
    /// An event handler to be called when the `close` button is clicked.
    pub on_close: Option<EventHandler<bool>>,
    /// A flag to determine whether the message is visible or not.
    #[props(default)]
    pub visible: bool,
    /// The title in the message header.
    #[props(into, default)]
    pub title: SharedString,
    /// A message to render when the future value is not ready.
    #[props(into, default)]
    pub loading: SharedString,
    /// A message to render when the future value is resolved.
    #[props(into, default)]
    pub success: SharedString,
    /// A message to render when the future value is rejected.
    #[props(into, default)]
    pub error: SharedString,
    /// An event handler to be called when the future value is not ready.
    pub on_loading: Option<EventHandler>,
    /// An event handler to be called when the future value is resolved.
    pub on_success: Option<EventHandler<T>>,
    /// An event handler to be called when the future value is rejected.
    pub on_error: Option<EventHandler<E>>,
}
