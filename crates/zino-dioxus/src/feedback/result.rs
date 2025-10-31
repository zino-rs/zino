use crate::class::Class;
use dioxus::prelude::*;
use std::time::Duration;
use zino_core::SharedString;

/// Global messages as feedback in response to user operations.
pub fn OperationResult<T, E>(props: OperationResultProps<T, E>) -> Element
where
    T: Clone + PartialEq + 'static,
    E: Clone + PartialEq + 'static,
{
    let mut visible = props.visible;
    if !visible() {
        return rsx! {};
    }

    let duration = Duration::from_millis(props.duration);
    match (props.future)() {
        Some(Ok(data)) => {
            spawn(async move {
                tokio::time::sleep(duration).await;
                visible.set(false);
                if let Some(handler) = props.on_success.as_ref() {
                    handler.call(data);
                }
            });
            rsx! {
                div {
                    class: "{props.class} is-success",
                    position: "fixed",
                    top: "4rem",
                    right: "0.75rem",
                    z_index: 9999,
                    if !props.title.is_empty() {
                        div {
                            class: "message-header",
                            span { { props.title.into_owned() } }
                            button {
                                r#type: "button",
                                class: props.close_class,
                                onclick: move |_event| {
                                    visible.set(false);
                                    if let Some(handler) = props.on_close.as_ref() {
                                        handler.call(false);
                                    }
                                }
                            }
                        }
                    }
                    div {
                        class: "message-body",
                        { props.success.into_owned() }
                    }
                }
            }
        }
        Some(Err(err)) => {
            spawn(async move {
                tokio::time::sleep(duration).await;
                visible.set(false);
                if let Some(handler) = props.on_error.as_ref() {
                    handler.call(err);
                }
            });
            rsx! {
                div {
                    class: "{props.class} is-danger",
                    position: "fixed",
                    top: "4rem",
                    right: "0.75rem",
                    z_index: 9999,
                    if !props.title.is_empty() {
                        div {
                            class: "message-header",
                            span { { props.title.into_owned() } }
                            button {
                                r#type: "button",
                                class: props.close_class,
                                onclick: move |_event| {
                                    visible.set(false);
                                    if let Some(handler) = props.on_close.as_ref() {
                                        handler.call(false);
                                    }
                                }
                            }
                        }
                    }
                    div {
                        class: "message-body",
                        span { { props.error.into_owned() } }
                    }
                }
            }
        }
        None => {
            if let Some(handler) = props.on_loading.as_ref() {
                handler.call(());
            }
            if props.loading.is_empty() {
                rsx! {}
            } else {
                rsx! {
                    div {
                        class: "{props.class} is-warning",
                        position: "fixed",
                        top: "4rem",
                        right: "0.75rem",
                        z_index: 99,
                        if !props.title.is_empty() {
                            div {
                                class: "message-header",
                                span { { props.title.into_owned() } }
                                button {
                                    r#type: "button",
                                    class: props.close_class,
                                    onclick: move |_event| {
                                        visible.set(false);
                                        if let Some(handler) = props.on_close.as_ref() {
                                            handler.call(false);
                                        }
                                    }
                                }
                            }
                        }
                        div {
                            class: "message-body",
                            span { { props.loading.into_owned() } }
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
    #[props(into, default = "message")]
    pub class: Class,
    /// A class to apply to the `close` button element.
    #[props(into, default = "delete")]
    pub close_class: Class,
    /// A flag to determine whether the message is visible or not.
    pub visible: Signal<bool>,
    /// A future value which represents the result of user operations.
    pub future: Resource<Result<T, E>>,
    /// A duration in milliseconds.
    #[props(default = 1500)]
    pub duration: u64,
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
    /// An event handler to be called when the `close` button is clicked.
    pub on_close: Option<EventHandler<bool>>,
}
