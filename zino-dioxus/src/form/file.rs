use crate::{class::Class, icon::SvgIcon};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaUpload;
use std::path::Path;
use zino_core::{file::NamedFile, SharedString};

/// A custom file upload input.
pub fn FileUpload(props: FileUploadProps) -> Element {
    let mut file_names = use_signal(Vec::new);
    let has_name = props.children.is_some() || !file_names().is_empty();
    rsx! {
        div {
            class: props.class,
            class: if !props.color.is_empty() { "is-{props.color}" },
            class: if !props.size.is_empty() { "is-{props.size}" },
            class: if props.fullwidth { "is-fullwidth" },
            class: if has_name { "has-name" },
            label {
                class: props.label_class.clone(),
                input {
                    class: props.input_class,
                    r#type: "file",
                    ..props.attributes,
                    onchange: move |event| async move {
                        if let Some(handler) = props.on_change.as_ref() {
                            if let Some(file_engine) = event.files() {
                                let mut files = Vec::new();
                                file_names.write().clear();
                                for file in file_engine.files() {
                                    if let Some(bytes) = file_engine.read_file(&file).await {
                                        let file_path = Path::new(&file);
                                        if let Some(file_name) = file_path
                                            .file_name()
                                            .map(|f| f.to_string_lossy())
                                        {
                                            let mut file = NamedFile::new(file_name);
                                            if let Some(file_name) = file.file_name() {
                                                file_names.write().push(file_name.to_owned());
                                            }
                                            file.set_bytes(bytes);
                                            files.push(file);
                                        }
                                    }
                                }
                                handler.call(files);
                            }
                        }
                    }
                }
                div {
                    class: "file-cta",
                    span {
                        class: "file-icon",
                        if props.icon.is_some() {
                            { props.icon }
                        } else {
                            SvgIcon {
                                shape: FaUpload,
                                width: 16,
                            }
                        }
                    }
                    span {
                        class: props.label_class,
                        { props.label }
                    }
                }
                if props.children.is_some() {
                    span {
                        class: "file-name",
                        { props.children }
                    }
                } else if !file_names().is_empty() {
                    span {
                        class: "file-name",
                        { file_names().join(", ") }
                    }
                }
            }
        }
    }
}

/// The [`FileUpload`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct FileUploadProps {
    /// The class attribute for the component.
    #[props(into, default = "file".into())]
    pub class: Class,
    /// A class to apply to the `label` element.
    #[props(into, default = "file-label".into())]
    pub label_class: Class,
    /// A class to apply to the `label` element.
    #[props(into, default = "file-input".into())]
    pub input_class: Class,
    /// A flag to determine whether the control is fullwidth or not.
    /// The color of the button: `primary` | `link` | `info` | `success` | `warning` | `danger`.
    #[props(into, default)]
    pub color: SharedString,
    /// The size of the button: `small` | `normal` | `medium` | `large`.
    #[props(into, default)]
    pub size: SharedString,
    #[props(default)]
    pub fullwidth: bool,
    /// The label content.
    #[props(into)]
    pub label: SharedString,
    /// An optional upload icon.
    pub icon: Option<VNode>,
    /// An event handler to be called when the files are selected.
    pub on_change: Option<EventHandler<Vec<NamedFile>>>,
    /// Spreading the props of the `input` element.
    #[props(extends = input)]
    attributes: Vec<Attribute>,
    /// The children to render within the component.
    children: Option<VNode>,
}
