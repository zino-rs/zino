use dioxus::prelude::*;
use zino_core::{json, SharedString};

/// A ToastUI Editor.
pub fn TuiEditor(props: TuiEditorProps) -> Element {
    let eval = eval(
        r#"
        let options = await dioxus.recv();
        options.el = document.getElementById(options.id);
        options.plugins = [toastui.Editor.plugin.codeSyntaxHighlight];
        const tuiEditor = new toastui.Editor(options);
        "#,
    );
    rsx! {
        div {
            id: "{props.id}",
            onmounted: move |_event| {
                let options = json!({
                    "id": props.id,
                    "height": props.height,
                    "minHeight": props.min_height,
                    "initialValue": props.content,
                    "initialEditType": props.edit_type,
                    "previewStyle": props.preview_style,
                    "language": props.locale,
                    "theme": props.theme,
                    "referenceDefinition": true,
                    "usageStatistics": false,
                });
                eval.send(options).ok();
            },
        }
    }
}

/// The [`TuiEditor`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct TuiEditorProps {
    /// The editor ID.
    #[props(into, default = "editor".into())]
    pub id: SharedString,
    /// The initial value of Markdown string.
    #[props(into)]
    pub content: SharedString,
    /// The height of the container.
    #[props(into, default = "auto".into())]
    pub height: SharedString,
    /// The min-height of the container.
    #[props(into, default = "300px".into())]
    pub min_height: SharedString,
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
}
