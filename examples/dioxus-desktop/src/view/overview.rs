use dioxus::prelude::*;

pub fn Overview(cx: Scope) -> Element {
    let data = [
        ("zino", "Application integrations."),
        ("zino-core", "Core types and traits."),
        ("zino-derive", "Derived traits."),
        ("zino-model", "Domain models."),
    ];
    render! {
        div {
            class: "columns is-6",
            div {
                class: "column",
                div {
                    class: "card",
                    header {
                        class: "card-header",
                        div {
                            class: "card-header-title",
                            "GitHub issues"
                        }
                    }
                    div {
                        class: "card-content",
                        img {
                            class: "mr-2",
                            src: "https://img.shields.io/github/issues/photino/zino",
                        }
                        img {
                            class: "mr-2",
                            src: "https://img.shields.io/github/issues/photino/zino/bug",
                        }
                        img {
                            class: "mr-2",
                            src: "https://img.shields.io/github/issues/photino/zino/enhancement",
                        }
                        img {
                            class: "mr-2",
                            src: "https://img.shields.io/github/issues/photino/zino/dependencies",
                        }
                    }
                }
            }
            div {
                class: "column",
                div {
                    class: "card",
                    header {
                        class: "card-header",
                        div {
                            class: "card-header-title",
                            "GitHub commits"
                        }
                    }
                    div {
                        class: "card-content",
                        img {
                            class: "mr-2",
                            src: "https://img.shields.io/github/commit-activity/t/photino/zino",
                        }
                        img {
                            class: "mr-2",
                            src: "https://img.shields.io/github/commit-activity/y/photino/zino",
                        }
                        img {
                            class: "mr-2",
                            src: "https://img.shields.io/github/commit-activity/m/photino/zino",
                        }
                        img {
                            class: "mr-2",
                            src: "https://img.shields.io/github/commit-activity/w/photino/zino",
                        }
                    }
                }
            }
        }
        div {
            class: "columns is-6",
            for d in data {
                div {
                    class: "column",
                    div {
                        class: "card",
                        header {
                            class: "card-header",
                            div {
                                class: "card-header-title",
                                span {
                                    class: "tag is-warning is-light mr-1",
                                    "{d.0}"
                                }
                                span { "{d.1}" }
                            }
                        }
                        div {
                            class: "card-content",
                            img {
                                class: "mr-2",
                                src: "https://img.shields.io/crates/v/{d.0}",
                            }
                            img {
                                class: "mr-2",
                                src: "https://shields.io/docsrs/{d.0}",
                            }
                            img {
                                class: "mr-2",
                                src: "https://img.shields.io/crates/l/{d.0}",
                            }
                            img {
                                class: "mr-2",
                                src: "https://img.shields.io/crates/d/{d.0}"
                            }
                            img {
                                class: "mr-2",
                                src: "https://img.shields.io/crates/dr/{d.0}"
                            }
                        }
                    }
                }
            }
        }
    }
}
