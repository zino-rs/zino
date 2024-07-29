use dioxus::prelude::*;
use std::time::Duration;
use zino_core::{json, JsonValue, SharedString};

/// A ToastUI Editor.
pub fn TuiEditor(props: TuiEditorProps) -> Element {
    let mut markdown = use_signal(String::new);
    let eval_editor = eval(
        r#"
        const { Editor } = toastui;
        const { codeSyntaxHighlight } = Editor.plugin;

        let options = await dioxus.recv();
        options.el = document.getElementById(options.id);
        options.plugins = [codeSyntaxHighlight];
        options.events = {
            change: function() {
                document.getElementById("tui-editor-input").value = tuiEditor.getMarkdown();
            },
        };
        const tuiEditor = new Editor(options);
        tuiEditor.show();
        "#,
    );
    spawn(async move {
        loop {
            let mut eval = eval(
                r#"
                const value = document.getElementById("tui-editor-input").value;
                dioxus.send(value);
                "#,
            );
            if let Ok(JsonValue::String(s)) = eval.recv().await {
                if markdown() != s {
                    if let Some(handler) = props.on_change.as_ref() {
                        handler.call(s.clone());
                    }
                    markdown.set(s);
                }
            }
            // Sleep for 100 milliseconds to avoid blocking the thread.
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    rsx! {
        input {
            id: "tui-editor-input",
            r#type: "hidden",
        }
        div {
            id: "{props.id}",
            onmounted: move |_event| {
                let options = json!({
                    "id": props.id,
                    "height": props.height,
                    "minHeight": props.min_height,
                    "initialValue": props.initial_value,
                    "initialEditType": props.edit_type,
                    "previewStyle": props.preview_style,
                    "language": props.locale,
                    "theme": props.theme,
                    "referenceDefinition": true,
                    "usageStatistics": false,
                });
                eval_editor.send(options).ok();
            }
        }
    }
}

/// The [`TuiEditor`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct TuiEditorProps {
    /// The editor ID.
    #[props(into, default = "editor".into())]
    pub id: SharedString,
    /// The height of the container.
    #[props(into, default = "auto".into())]
    pub height: SharedString,
    /// The min-height of the container.
    #[props(into, default = "300px".into())]
    pub min_height: SharedString,
    /// The initial value of Markdown string.
    #[props(into)]
    pub initial_value: SharedString,
    /// The initial type to show: `markdown` | `wysiwyg`.
    #[props(into, default = "markdown".into())]
    pub edit_type: SharedString,
    /// The preview style of Markdown mode: `tab` | `vertical`.
    #[props(into, default = "vertical".into())]
    pub preview_style: SharedString,
    /// The theme: `light` | `dark`.
    #[props(into, default = "light".into())]
    pub theme: SharedString,
    /// The i18n locale.
    #[props(into, default = "en-US".into())]
    pub locale: SharedString,
    /// An event handler to be called when the input value is changed.
    pub on_change: Option<EventHandler<String>>,
}
