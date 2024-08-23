use crate::{class::Class, icon::SvgIcon};
use dioxus::prelude::*;
use dioxus_free_icons::icons::{bs_icons::*, fa_solid_icons::FaUpload};
use std::{
    fs,
    path::{Path, PathBuf},
};
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

/// A list of files and folders in a hierarchical tree structure.
pub fn FileTree(props: FileTreeProps) -> Element {
    let mut opened = use_signal(|| props.opened);
    let tree_id = props.tree_id;
    let icon_size = props.icon_size;
    let show_file_icons = props.show_file_icons;
    let on_click = props.on_click;
    let current_dir = props.current_dir.as_ref()?;
    let current_dir_name = current_dir.file_name().and_then(|s| s.to_str())?;
    let mut folders = Vec::new();
    let mut files = Vec::new();
    if opened() {
        let entries = fs::read_dir(current_dir).ok()?;
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let path = entry.path();
                if metadata.is_dir() {
                    folders.push(path);
                } else if metadata.is_file() {
                    if let Some(name) = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|s| s.to_owned())
                    {
                        let extension = path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|s| s.to_ascii_lowercase());
                        files.push((name, extension, path));
                    }
                }
            }
        }
    }
    rsx! {
        div {
            key: "{tree_id}-{current_dir_name}",
            class: "{props.class}",
            cursor: "pointer",
            div {
                onclick: move |_event| {
                    opened.set(!opened());
                },
                if opened() {
                    SvgIcon {
                        shape: BsChevronDown,
                        width: icon_size,
                    }
                } else {
                    SvgIcon {
                        shape: BsChevronRight,
                        width: icon_size,
                    }
                }
                span { "{current_dir_name}" }
            }
            for path in folders {
                FileTree {
                    class: props.class.clone(),
                    file_class: props.file_class.clone(),
                    current_dir: Some(path),
                    tree_id: tree_id.clone(),
                    icon_size: icon_size,
                    show_file_icons: show_file_icons,
                    opened: false,
                    on_click: on_click,
                }
            }
            for (name, extension, path) in files {
                div {
                    class: "{props.file_class}",
                    onclick: move |_event| {
                        if let Some(handler) = on_click.as_ref() {
                            handler.call(path.clone());
                        }
                    },
                    if show_file_icons {
                        FileIcon {
                            extension: extension,
                            icon_size: icon_size,
                        }
                    }
                    span { "{name}" }
                }
            }
        }
    }
}

/// The [`FileTree`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct FileTreeProps {
    /// The class attribute for the component.
    #[props(into, default = "file-tree".into())]
    pub class: Class,
    /// The class attribute for files in the current directory.
    #[props(into, default = "file-node".into())]
    pub file_class: Class,
    /// The current directory.
    pub current_dir: Option<PathBuf>,
    /// An identifer of the tree.
    pub tree_id: String,
    /// The size of the icon.
    #[props(default = 16)]
    pub icon_size: u32,
    /// A flag to indicate whether the file icons are shown.
    #[props(default)]
    pub show_file_icons: bool,
    /// A flag to indicate whether the directory is opened or not.
    #[props(default)]
    pub opened: bool,
    /// An event handler to be called when a file is clicked.
    pub on_click: Option<EventHandler<PathBuf>>,
}

/// An icon for different file extensions.
pub fn FileIcon(props: FileIconProps) -> Element {
    let icon_size = props.icon_size;
    let extension = props.extension.unwrap_or_default();
    match extension.as_str() {
        "js" | "py" | "sql" | "rs" | "wino" => rsx!(SvgIcon {
            shape: BsFileCode,
            width: icon_size,
        }),
        "xls" | "xlsx" => rsx!(SvgIcon {
            shape: BsFileExcel,
            width: icon_size,
        }),
        "ico" | "gif" | "jpg" | "jpeg" | "png" | "svg" | "webp" => rsx!(SvgIcon {
            shape: BsFileImage,
            width: icon_size,
        }),
        "pdf" => rsx!(SvgIcon {
            shape: BsFilePdf,
            width: icon_size,
        }),
        "doc" | "docx" => rsx!(SvgIcon {
            shape: BsFileWord,
            width: icon_size,
        }),
        _ => rsx!(SvgIcon {
            shape: BsFileText,
            width: icon_size,
        }),
    }
}

/// The [`FileIcon`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct FileIconProps {
    /// The file extension.
    pub extension: Option<String>,
    /// The size of the icon.
    #[props(default = 16)]
    pub icon_size: u32,
}
