use crate::service;
use dioxus::prelude::*;
use zino::prelude::*;

pub fn StargazerList(cx: Scope) -> Element {
    let stargazers = use_future(cx, (), |_| service::stargazer::list_stargazers(10, 1));
    match stargazers.value() {
        Some(Ok(items)) => {
            render! {
                div {
                    class: "columns",
                    div {
                        class: "column",
                        img {
                            width: 800,
                            height: 533,
                            src: "https://api.star-history.com/svg?repos=photino/zino&type=Timeline"
                        }
                    }
                    div {
                        class: "column",
                        table {
                            class: "table is-fullwidth",
                            thead {
                                tr {
                                    th {}
                                    th { "Account" }
                                    th { "Avatar" }
                                }
                            }
                            tbody {
                                for (index, item) in items.iter().enumerate() {
                                    StargazerListing {
                                        index: index + 1,
                                        stargazer: item.clone(),
                                    }
                                }
                            }
                        }
                        nav {
                            class: "pagination is-centered",
                            a {
                                class: "pagination-previous",
                                "Previous"
                            }
                            a {
                                class: "pagination-next",
                                "Next"
                            }
                        }
                    }
                }
            }
        }
        Some(Err(err)) => {
            render! { "An error occurred while fetching stargazers: {err}" }
        }
        None => {
            render! {
                progress {
                    class: "progress is-small is-primary",
                    max: 100,
                }
            }
        }
    }
}

#[inline_props]
fn StargazerListing(cx: Scope, index: usize, stargazer: Map) -> Element {
    let name = stargazer.get_str("login").unwrap_or("N/A");
    let avatar_url = stargazer.get_str("avatar_url").unwrap();
    render! {
        tr {
            th { "{index}" }
            td {
                span { "{name}" }
            }
            td {
                figure {
                   class: "image is-24x24",
                   img { src: avatar_url }
                }
            }
        }
    }
}
