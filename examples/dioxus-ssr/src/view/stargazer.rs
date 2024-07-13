use crate::service;
use dioxus::prelude::*;
use zino::prelude::*;
use zino_dioxus::prelude::*;

pub fn StargazerList() -> Element {
    let stargazers_count = use_resource(service::stargazer::get_stargazers_count);
    match &*stargazers_count.value().read_unchecked() {
        Some(Ok(count)) => {
            rsx! {
                div {
                    class: "columns is-desktop is-6",
                    div {
                        class: "column",
                        StargazerHistory {}
                    }
                    div {
                        class: "column",
                        StargazerListTable { num_stargazers: *count }
                    }
                }
            }
        }
        Some(Err(err)) => {
            rsx! {
                div {
                    class: "notification is-danger is-light",
                    "An error occurred while fetching stargazers count: {err}"
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

fn StargazerHistory() -> Element {
    let mut chart_type = use_signal(|| "Date");
    rsx! {
        label {
            class: "checkbox is-pulled-right",
            input {
                r#type: "checkbox",
                onchange: move |event| {
                    let value = if event.value() == "true" { "Timeline" } else { "Date" };
                    chart_type.set(value);
                },
            }
            span {
                class: "ml-1",
                "Align timeline"
            }
        }
        img {
            margin_top: "-30px",
            width: 800,
            height: 533,
            src: "https://api.star-history.com/svg?repos=zino-rs/zino&type={chart_type}",
        }
    }
}

#[component]
fn StargazerListTable(num_stargazers: usize) -> Element {
    let mut current_page = use_signal(|| 1);
    let stargazers = use_resource(move || async move {
        service::stargazer::list_stargazers(10, current_page()).await
    });
    match &*stargazers.value().read_unchecked() {
        Some(Ok(stargazers)) => {
            rsx! {
                table {
                    class: "table is-fullwidth",
                    thead {
                        tr {
                            th {}
                            th { "Username" }
                            th { "Avatar" }
                            th { "Followers" }
                            th { "Starred at" }
                        }
                    }
                    tbody {
                        for (index, stargazer) in stargazers.iter().enumerate() {
                            StargazerItem {
                                index: (current_page - 1) * 10 + index + 1,
                                stargazer: stargazer.clone(),
                            }
                        }
                    }
                }
                Pagination {
                    total: num_stargazers,
                    current_page: current_page(),
                    on_change: move |page| {
                        current_page.set(page);
                    }
                }
            }
        }
        Some(Err(err)) => {
            rsx! {
                div {
                    class: "notification is-danger is-light",
                    "An error occurred while fetching stargazers: {err}"
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
fn StargazerItem(index: usize, stargazer: Map) -> Element {
    let name = stargazer.get_str("login").unwrap_or_default();
    let avatar_url = stargazer.get_str("avatar_url").unwrap_or_default();
    let starred_at = stargazer.get_str("starred_at").unwrap_or_default();
    rsx! {
        tr {
            th { "{index}" }
            td {
                a {
                    href: "https://github.com/{name}",
                    "{name}"
                }
            }
            td {
                figure {
                   class: "image is-24x24",
                   img { src: avatar_url },
                }
            }
            td {
                a {
                    href: "https://github.com/{name}?tab=followers",
                    img {
                        src: "https://img.shields.io/github/followers/{name}?label=&style=social"
                    }
                }
            }
            td {
                span { "{starred_at}" }
            }
        }
    }
}
