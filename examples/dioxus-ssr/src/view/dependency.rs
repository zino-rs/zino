use crate::service;
use dioxus::prelude::*;
use zino::prelude::*;

pub fn DependencyList() -> Element {
    let dependencies = use_resource(service::dependency::list_dependencies);
    match &*dependencies.value().read_unchecked() {
        Some(Ok(items)) => {
            rsx! {
                table {
                    class: "table is-fullwidth",
                    thead {
                        tr {
                            th {
                                a {
                                    href: "https://docs.rs/crate/zino-core/latest/source/Cargo.toml",
                                    "zino-core/Cargo.toml"
                                }
                            }
                            th { "Requirements" }
                            th { "Latest stable" }
                            th { "Latest release" }
                            th { "Licenses" }
                            th { "Downloads" }
                        }
                    }
                    tbody {
                        for item in items {
                            DependencyListing { dep: item.clone() }
                        }
                    }
                }
            }
        }
        Some(Err(err)) => {
            rsx! {
                div {
                    class: "notification is-danger is-light",
                    "An error occurred while fetching dependencies: {err}"
                }
            }
        }
        None => {
            rsx! {
                progress {
                    class: "progress is-small is-primary",
                    max: 100,
                }
            }
        }
    }
}

#[component]
fn DependencyListing(dep: Map) -> Element {
    let name = dep.get_str("name").unwrap_or_default();
    let requirements = dep.get_str("requirements").unwrap_or_default();
    let latest_stable = dep.get_str("latest_stable").unwrap_or_default();
    let latest_release = dep.get_str("latest").unwrap_or_default();
    let licenses = dep.get_str_array("normalized_licenses").unwrap_or_default();
    let latest_release_tag_class = if latest_release != latest_stable {
        "is-warning"
    } else {
        "is-info"
    };
    let requirements_tag_class = if dep.get_bool("outdated") == Some(true) {
        "is-danger"
    } else {
        "is-success"
    };
    rsx! {
        tr {
            td {
                a {
                    href: "https://crates.io/crates/{name}",
                    "{name}"
                }
            }
            td {
                span {
                    class: "tag {requirements_tag_class} is-light",
                    "{requirements}"
                }
            }
            td {
                a {
                    href: "https://docs.rs/{name}/{latest_stable}",
                    span {
                        class: "tag is-link is-light",
                        "{latest_stable}"
                    }
                }
            }
            td {
                span {
                    class: "tag {latest_release_tag_class} is-light",
                    "{latest_release}"
                }
            }
            td {
                for license in licenses {
                    span {
                        class: "tag is-info is-light mr-1",
                        "{license}"
                    }
                }
            }
            td {
                img {
                    class: "mr-1",
                    src: "https://img.shields.io/crates/d/{name}?label=all"
                }
                img {
                    src: "https://img.shields.io/crates/dr/{name}?label=recent"
                }
            }
        }
    }
}
