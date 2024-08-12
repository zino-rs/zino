use crate::class::Class;
use comrak::{
    plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder},
    Options, Plugins,
};
use dioxus::prelude::*;
use zino_core::{LazyLock, SharedString};

/// A markdown-to-html converter.
pub fn Markdown(props: MarkdownProps) -> Element {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.tagfilter = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.superscript = true;
    options.extension.footnotes = true;
    options.extension.description_lists = true;
    options.extension.math_dollars = true;
    options.extension.shortcodes = true;
    options.extension.underline = true;
    options.parse.smart = true;
    options.parse.relaxed_autolinks = true;
    options.render.full_info_string = true;
    options.render.escape = true;
    options.render.ignore_setext = true;
    options.render.ignore_empty_links = true;

    let text = props.content.as_ref();
    let html = comrak::markdown_to_html_with_plugins(text, &options, &COMRAK_PLUGINS);
    rsx! {
        div {
            class: props.class,
            dangerous_inner_html: html,
        }
    }
}

/// The [`Markdown`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct MarkdownProps {
    /// The class attribute for the component.
    #[props(into, default = "content markdown".into())]
    pub class: Class,
    /// The children to render within the component.
    #[props(into)]
    pub content: SharedString,
}

/// Default comrak plugins.
static COMRAK_PLUGINS: LazyLock<Plugins> = LazyLock::new(|| {
    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&*SYNTECT_ADAPTER);
    plugins
});

/// Syntect adapter.
static SYNTECT_ADAPTER: LazyLock<SyntectAdapter> =
    LazyLock::new(|| SyntectAdapterBuilder::new().build());
