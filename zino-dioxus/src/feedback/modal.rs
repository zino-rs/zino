use crate::class::Class;
use dioxus::prelude::*;
use zino_core::SharedString;

/// A classic modal with a header and a body.
pub fn ModalCard(props: ModalCardProps) -> Element {
    let size = match props.size.as_ref() {
        "small" => 25,
        "medium" => 50,
        "large" => 75,
        _ => 40,
    };
    rsx! {
        div {
            class: props.class,
            class: if props.visible { props.active_class },
            style: "--bulma-modal-content-width:{size}rem",
            div { class: "modal-background" }
            div {
                class: "modal-card",
                header {
                    class: "modal-card-head",
                    div {
                        class: "modal-card-title {props.title_class}",
                        { props.title }
                    }
                    button {
                        r#type: "button",
                        class: props.close_class,
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
    /// The size of the modal: `small` | `normal` | `medium` | `large`.
    #[props(into, default)]
    pub size: SharedString,
    /// The title in the modal header.
    #[props(into)]
    pub title: SharedString,
    /// The modal body to render within the component.
    children: Element,
}

/// A dynamic data type for the modal.
#[derive(Clone, Default, PartialEq)]
pub struct ModalData<T> {
    /// A optional ID.
    id: Option<T>,
    /// A optional name.
    name: SharedString,
    /// The title in the modal header.
    title: SharedString,
    /// A flag to determine whether the modal is visible or not.
    visible: bool,
}

impl<T> ModalData<T> {
    /// Creates a new instance.
    #[inline]
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            id: None,
            name: "modal".into(),
            title: title.into(),
            visible: false,
        }
    }

    /// Sets the id.
    #[inline]
    pub fn set_id(&mut self, id: T) {
        self.id = Some(id);
    }

    /// Sets the name.
    #[inline]
    pub fn set_name(&mut self, name: impl Into<SharedString>) {
        self.name = name.into();
    }

    /// Sets the `title` property.
    #[inline]
    pub fn set_title(&mut self, title: impl Into<SharedString>) {
        self.title = title.into();
    }

    /// Sets the `visible` property.
    #[inline]
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Returns a reference to the optional ID.
    #[inline]
    pub fn id(&self) -> Option<&T> {
        self.id.as_ref()
    }

    /// Returns a reference to the name.
    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Returns the `title` property.
    #[inline]
    pub fn title(&self) -> String {
        self.title.as_ref().to_owned()
    }

    /// Returns the `visible` property.
    #[inline]
    pub fn visible(&self) -> bool {
        self.visible
    }
}
