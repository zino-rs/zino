use dioxus::prelude::*;
use dioxus_sdk::clipboard::use_clipboard;
use zino_core::SharedString;

/// A button to copy the content to clipboard when clicked.
pub fn CopyToClipboard(props: CopyToClipboardProps) -> Element {
    if props.hidden {
        return None;
    }

    let mut clipboard = use_clipboard();
    rsx! {
        a {
            onclick: move |event| {
                let content = props.content.as_ref();
                match clipboard.set(content.to_owned()) {
                    Ok(_) => if let Some(handler) = props.on_success.as_ref() {
                        handler.call(());
                    },
                    Err(_) => if let Some(handler) = props.on_error.as_ref() {
                        handler.call(());
                    }
                }
                event.stop_propagation();
            },
            { props.children }
        }
    }
}

/// The [`CopyToClipboard`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct CopyToClipboardProps {
    /// A flag to determine whether the button is hidden or not.
    #[props(default)]
    pub hidden: bool,
    /// The content value to be copied to clipboard.
    #[props(into)]
    pub content: SharedString,
    /// An event handler to be called when the content value is copied to clipboard.
    pub on_success: Option<EventHandler<()>>,
    /// An event handler to be called when the content value is failed to copy to clipboard.
    pub on_error: Option<EventHandler<()>>,
    /// The children to render within the component.
    children: Element,
}
